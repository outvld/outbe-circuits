// SPDX-License-Identifier: Apache-2.0
package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net"
	"net/http"
	"net/url"
	"os"
	"strings"
	"time"
)

const maxRequestBytes = 64 << 10
const processPolicy = "resident artifacts initialized once at server startup; one native request at a time"
const cancellationPolicy = "client cancellation stops waiting; active native computation finishes before the server accepts another request"

type wrapperService struct {
	session *session
	busy    chan struct{}
}

func newWrapperService(keys string) (*wrapperService, error) {
	session, err := loadSession(keys, true)
	if err != nil {
		return nil, err
	}
	return &wrapperService{session: session, busy: make(chan struct{}, 1)}, nil
}

func apiError(w http.ResponseWriter, status int, err error) {
	message := err.Error()
	if len(message) > 4096 {
		message = message[:4096]
	}
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Cache-Control", "no-store")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(map[string]any{"verified": false, "error": message})
}

func (s *wrapperService) metadata(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		w.Header().Set("Allow", "GET")
		apiError(w, http.StatusMethodNotAllowed, errors.New("GET required"))
		return
	}
	if s.session == nil {
		apiError(w, http.StatusServiceUnavailable, errors.New("native wrapper is not initialized"))
		return
	}
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Cache-Control", "no-store")
	_ = json.NewEncoder(w).Encode(map[string]any{
		"available": true, "circuit": "demo_tribute", "inner_vk_sha256": innerVKHash,
		"backend": "native-go-gnark", "process_policy": processPolicy, "cancellation_policy": cancellationPolicy,
		"setup_provenance": s.session.setup, "initialization": s.session.initialization,
	})
}

func sameOrigin(r *http.Request) bool {
	// Origin is absent for command-line clients; browsers supply it on POST.
	origins := r.Header.Values("Origin")
	if len(origins) == 0 {
		return r.Header.Get("Sec-Fetch-Site") != "cross-site"
	}
	if len(origins) != 1 {
		return false
	}
	u, err := url.Parse(origins[0])
	scheme := "http"
	if r.TLS != nil {
		scheme = "https"
	}
	return err == nil && u.Scheme == scheme && strings.EqualFold(u.Host, r.Host) && u.User == nil && u.Path == "" && u.RawQuery == "" && u.Fragment == ""
}

func (s *wrapperService) heavy(command string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		requestStart := time.Now()
		if r.Method != http.MethodPost {
			w.Header().Set("Allow", "POST")
			apiError(w, http.StatusMethodNotAllowed, errors.New("POST required"))
			return
		}
		if !sameOrigin(r) {
			apiError(w, http.StatusForbidden, errors.New("cross-origin POST rejected"))
			return
		}
		r.Body = http.MaxBytesReader(w, r.Body, maxRequestBytes)
		data, err := io.ReadAll(r.Body)
		if err != nil {
			status := http.StatusBadRequest
			var tooLarge *http.MaxBytesError
			if errors.As(err, &tooLarge) {
				status = http.StatusRequestEntityTooLarge
			}
			apiError(w, status, err)
			return
		}
		var req request
		if err := json.Unmarshal(data, &req); err != nil {
			apiError(w, http.StatusBadRequest, err)
			return
		}
		if _, _, _, err := publicAssignment(req); err != nil {
			apiError(w, http.StatusBadRequest, err)
			return
		}
		if command == "prove" {
			_, err = hexData(req.ProofHex, proofBytes)
		} else {
			var proof []byte
			proof, err = hexData(req.Groth16ProofHex, -1)
			if err == nil && len(proof) == 0 {
				err = errors.New("Groth16 proof required")
			}
		}
		if err != nil {
			apiError(w, http.StatusBadRequest, err)
			return
		}
		if err := r.Context().Err(); err != nil {
			apiError(w, http.StatusRequestTimeout, err)
			return
		}
		select {
		case s.busy <- struct{}{}:
			defer func() { <-s.busy }()
		default:
			apiError(w, http.StatusConflict, errors.New("native wrapper is busy"))
			return
		}
		// Keep the slot until the native operation returns, even if the client
		// disconnects. The pinned gnark API cannot cancel active computation.
		response, err := s.session.execute(r.Context(), req, command == "verify")
		if r.Context().Err() != nil {
			return // Discard the result; do not pretend native work was canceled.
		}
		if err != nil {
			apiError(w, http.StatusUnprocessableEntity, err)
			return
		}
		response.RequestWallMS = ms(requestStart)
		w.Header().Set("Content-Type", "application/json")
		w.Header().Set("Cache-Control", "no-store")
		_ = json.NewEncoder(w).Encode(response)
	}
}

func serve(keys, web, listen string) error {
	info, err := os.Stat(web)
	if err != nil {
		return err
	}
	if !info.IsDir() {
		return errors.New("--web must be a directory")
	}
	// Reserve IPv4 before loading artifacts: an existing IPv4 service must
	// fail immediately rather than being shadowed by a wildcard IPv6 socket.
	listener, err := net.Listen("tcp4", listen)
	if err != nil {
		return err
	}
	defer listener.Close()
	s, err := newWrapperService(keys)
	if err != nil {
		return fmt.Errorf("initialize native wrapper: %w", err)
	}
	mux := http.NewServeMux()
	mux.HandleFunc("/api/wrapper", s.metadata)
	mux.HandleFunc("/api/wrap", s.heavy("prove"))
	mux.HandleFunc("/api/verify", s.heavy("verify"))
	mux.HandleFunc("/api/", func(w http.ResponseWriter, r *http.Request) {
		apiError(w, http.StatusNotFound, errors.New("unknown API endpoint"))
	})
	mux.Handle("/", http.FileServer(http.Dir(web)))
	server := &http.Server{Addr: listen, Handler: mux, ReadHeaderTimeout: 10 * time.Second, ReadTimeout: 30 * time.Second, IdleTimeout: 60 * time.Second, MaxHeaderBytes: 16 << 10}
	fmt.Fprintf(os.Stderr, "native wrapper listening on %s; static web %s; %s\n", listen, web, processPolicy)
	return server.Serve(listener)
}
