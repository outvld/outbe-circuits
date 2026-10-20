// SPDX-License-Identifier: Apache-2.0
package main

import (
	"context"
	"encoding/json"
	"errors"
	"net"
	"net/http"
	"net/http/httptest"
	"os"
	"reflect"
	"strings"
	"sync"
	"syscall"
	"testing"
	"time"
)

func requestJSON(t *testing.T, req request) string {
	t.Helper()
	data, err := json.Marshal(req)
	if err != nil {
		t.Fatal(err)
	}
	return string(data)
}

func TestServiceBoundaries(t *testing.T) {
	keys, req, _ := lifecycleSetup(t)
	s, err := newWrapperService(keys)
	if err != nil {
		t.Fatal(err)
	}
	proof, err := s.session.execute(context.Background(), req, false)
	if err != nil {
		t.Fatal(err)
	}
	req.Groth16ProofHex = proof.Groth16ProofHex
	valid := requestJSON(t, req)
	for _, test := range []struct {
		name, method, body, origin string
		fetchSite                  string
		busy                       bool
		status                     int
	}{
		{name: "method", method: "GET", body: valid, status: 405},
		{name: "invalid-json", method: "POST", body: "{", status: 400},
		{name: "trailing-json", method: "POST", body: valid + " {}", status: 400},
		{name: "invalid-statement", method: "POST", body: "{}", status: 400},
		{name: "cross-origin", method: "POST", body: valid, origin: "http://evil.example", status: 403},
		{name: "null-origin", method: "POST", body: valid, origin: "null", status: 403},
		{name: "cross-site", method: "POST", body: valid, fetchSite: "cross-site", status: 403},
		{name: "body-limit", method: "POST", body: strings.Repeat(" ", maxRequestBytes+1), status: 413},
		{name: "busy", method: "POST", body: valid, busy: true, status: 409},
		{name: "same-origin", method: "POST", body: valid, origin: "http://example.com", status: 200},
		{name: "cli-no-origin", method: "POST", body: valid, status: 200},
	} {
		t.Run(test.name, func(t *testing.T) {
			if test.busy {
				s.busy <- struct{}{}
				defer func() { <-s.busy }()
			}
			r := httptest.NewRequest(test.method, "http://example.com/api/verify", strings.NewReader(test.body))
			if test.origin != "" {
				r.Header.Set("Origin", test.origin)
			}
			if test.fetchSite != "" {
				r.Header.Set("Sec-Fetch-Site", test.fetchSite)
			}
			w := httptest.NewRecorder()
			s.heavy("verify")(w, r)
			if w.Code != test.status {
				t.Fatalf("status %d: %s", w.Code, w.Body.String())
			}
			if test.status == 200 {
				var res result
				if err := json.Unmarshal(w.Body.Bytes(), &res); err != nil {
					t.Fatal(err)
				}
				assertRequestMetrics(t, res, s.session.initialization, false)
			} else {
				var failure struct {
					Verified bool   `json:"verified"`
					Error    string `json:"error"`
				}
				if err := json.Unmarshal(w.Body.Bytes(), &failure); err != nil || failure.Verified || failure.Error == "" {
					t.Fatalf("invalid error response: %s", w.Body.String())
				}
			}
		})
	}
}

func assertMetadata(t *testing.T, s *wrapperService) {
	t.Helper()
	w := httptest.NewRecorder()
	s.metadata(w, httptest.NewRequest(http.MethodGet, "http://example.com/api/wrapper", nil))
	if w.Code != http.StatusOK {
		t.Fatalf("metadata %d: %s", w.Code, w.Body.String())
	}
	var meta struct {
		Available          bool           `json:"available"`
		Backend            string         `json:"backend"`
		ProcessPolicy      string         `json:"process_policy"`
		CancellationPolicy string         `json:"cancellation_policy"`
		Initialization     initialization `json:"initialization"`
		Setup              provenance     `json:"setup_provenance"`
	}
	if err := json.Unmarshal(w.Body.Bytes(), &meta); err != nil {
		t.Fatal(err)
	}
	if !meta.Available || meta.Backend != "native-go-gnark" || meta.ProcessPolicy != processPolicy || meta.CancellationPolicy != cancellationPolicy || !reflect.DeepEqual(meta.Initialization, s.session.initialization) || !reflect.DeepEqual(meta.Setup, s.session.setup) {
		t.Fatalf("invalid resident metadata: %s", w.Body.String())
	}
}

func TestResidentHTTPUsesRetainedArtifacts(t *testing.T) {
	keys, req, _ := lifecycleSetup(t)
	s, err := newWrapperService(keys)
	if err != nil {
		t.Fatal(err)
	}
	assertMetadata(t, s)
	if err := os.RemoveAll(keys); err != nil {
		t.Fatal(err)
	}
	var previous string
	for range 2 {
		assertMetadata(t, s)
		w := httptest.NewRecorder()
		s.heavy("prove")(w, httptest.NewRequest(http.MethodPost, "http://example.com/api/wrap", strings.NewReader(requestJSON(t, req))))
		if w.Code != http.StatusOK {
			t.Fatalf("wrap %d: %s", w.Code, w.Body.String())
		}
		var res result
		if err := json.Unmarshal(w.Body.Bytes(), &res); err != nil {
			t.Fatal(err)
		}
		assertRequestMetrics(t, res, s.session.initialization, true)
		if res.Groth16ProofHex == previous {
			t.Fatal("HTTP prove reused a proof")
		}
		previous = res.Groth16ProofHex
		verifyReq := req
		verifyReq.Groth16ProofHex = res.Groth16ProofHex
		w = httptest.NewRecorder()
		s.heavy("verify")(w, httptest.NewRequest(http.MethodPost, "http://example.com/api/verify", strings.NewReader(requestJSON(t, verifyReq))))
		if w.Code != http.StatusOK {
			t.Fatalf("verify %d: %s", w.Code, w.Body.String())
		}
		var verified result
		if err := json.Unmarshal(w.Body.Bytes(), &verified); err != nil {
			t.Fatal(err)
		}
		assertRequestMetrics(t, verified, s.session.initialization, false)
	}
	assertMetadata(t, s)
}

func TestServiceCancellationRetainsSlotUntilNativeCompletion(t *testing.T) {
	keys, req, _ := lifecycleSetup(t)
	s, err := newWrapperService(keys)
	if err != nil {
		t.Fatal(err)
	}
	started, release := make(chan struct{}), make(chan struct{})
	var once sync.Once
	releaseNative := func() { once.Do(func() { close(release) }) }
	lifecycleGate.Lock()
	lifecycleGate.started, lifecycleGate.release = started, release
	lifecycleGate.Unlock()
	defer func() {
		releaseNative()
		lifecycleGate.Lock()
		lifecycleGate.started, lifecycleGate.release = nil, nil
		lifecycleGate.Unlock()
	}()
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	r := httptest.NewRequest(http.MethodPost, "http://example.com/api/wrap", strings.NewReader(requestJSON(t, req))).WithContext(ctx)
	w := httptest.NewRecorder()
	done := make(chan struct{})
	go func() { s.heavy("prove")(w, r); close(done) }()
	select {
	case <-started:
	case <-time.After(5 * time.Second):
		t.Fatal("native prover did not start")
	}
	cancel()
	select {
	case <-done:
		t.Fatal("HTTP cancellation released a running native operation")
	default:
	}
	if len(s.busy) != 1 {
		t.Fatal("canceled native request lost its admission slot")
	}
	busy := httptest.NewRecorder()
	s.heavy("prove")(busy, httptest.NewRequest(http.MethodPost, "http://example.com/api/wrap", strings.NewReader(requestJSON(t, req))))
	if busy.Code != http.StatusConflict {
		t.Fatalf("concurrent native request: %d", busy.Code)
	}
	assertMetadata(t, s)
	releaseNative()
	select {
	case <-done:
	case <-time.After(5 * time.Second):
		t.Fatal("native operation did not finish after release")
	}
	if len(s.busy) != 0 || w.Body.Len() != 0 {
		t.Fatalf("finished disconnected request retained slot or wrote a result: slots=%d body=%q", len(s.busy), w.Body.String())
	}
	lifecycleGate.Lock()
	lifecycleGate.started, lifecycleGate.release = nil, nil
	lifecycleGate.Unlock()
	// A subsequent real proof succeeds with the same initialized state.
	next := httptest.NewRecorder()
	s.heavy("prove")(next, httptest.NewRequest(http.MethodPost, "http://example.com/api/wrap", strings.NewReader(requestJSON(t, req))))
	if next.Code != http.StatusOK {
		t.Fatalf("post-cancellation proof %d: %s", next.Code, next.Body.String())
	}
	var res result
	if err := json.Unmarshal(next.Body.Bytes(), &res); err != nil {
		t.Fatal(err)
	}
	assertRequestMetrics(t, res, s.session.initialization, true)
}

func TestServiceRejectsCanceledRequestBeforeNativeWork(t *testing.T) {
	keys, req, _ := lifecycleSetup(t)
	s, err := newWrapperService(keys)
	if err != nil {
		t.Fatal(err)
	}
	ctx, cancel := context.WithCancel(context.Background())
	cancel()
	r := httptest.NewRequest(http.MethodPost, "http://example.com/api/wrap", strings.NewReader(requestJSON(t, req))).WithContext(ctx)
	w := httptest.NewRecorder()
	s.heavy("prove")(w, r)
	if w.Code != http.StatusRequestTimeout || len(s.busy) != 0 {
		t.Fatalf("canceled request admitted: status=%d slots=%d", w.Code, len(s.busy))
	}
	if _, err := s.session.execute(ctx, req, false); !errors.Is(err, context.Canceled) {
		t.Fatalf("core admitted canceled native work: %v", err)
	}
}

func TestMetadataRejectsUninitializedService(t *testing.T) {
	s := &wrapperService{}
	w := httptest.NewRecorder()
	s.metadata(w, httptest.NewRequest(http.MethodGet, "http://example.com/api/wrapper", nil))
	if w.Code != http.StatusServiceUnavailable || strings.Contains(w.Body.String(), `"available":true`) {
		t.Fatalf("uninitialized service advertised: %d %s", w.Code, w.Body.String())
	}
}

func TestServerInitializationFailureClosesListener(t *testing.T) {
	listener, err := net.Listen("tcp4", "127.0.0.1:0")
	if err != nil {
		t.Fatal(err)
	}
	address := listener.Addr().String()
	if err := listener.Close(); err != nil {
		t.Fatal(err)
	}
	if err := serve(t.TempDir(), t.TempDir(), address); err == nil || !strings.Contains(err.Error(), "initialize native wrapper") {
		t.Fatalf("missing setup did not fail startup: %v", err)
	}
	listener, err = net.Listen("tcp4", address)
	if err != nil {
		t.Fatalf("failed initialization retained listener: %v", err)
	}
	listener.Close()
}

func TestServerRejectsOccupiedIPv4Port(t *testing.T) {
	listener, err := net.Listen("tcp4", "0.0.0.0:0")
	if err != nil {
		t.Fatal(err)
	}
	defer listener.Close()
	_, port, err := net.SplitHostPort(listener.Addr().String())
	if err != nil {
		t.Fatal(err)
	}
	keys, web := t.TempDir(), t.TempDir()
	done := make(chan error, 1)
	go func() { done <- serve(keys, web, net.JoinHostPort("0.0.0.0", port)) }()
	select {
	case err := <-done:
		if !errors.Is(err, syscall.EADDRINUSE) {
			t.Fatalf("expected occupied-port error, got %v", err)
		}
	case <-time.After(2 * time.Second):
		t.Fatal("server accepted a shadowed listener instead of rejecting the occupied IPv4 port")
	}
}
