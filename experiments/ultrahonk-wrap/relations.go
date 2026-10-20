// SPDX-License-Identifier: Apache-2.0
// Copyright 2022 Aztec. Modified: direct gnark constraint implementation.
// Formulas ported from Aztec's exact BB5 UltraKeccakZK Solidity verifier.
package main

const (
	qm = iota
	qc
	ql
	qr
	qo
	q4
	qlookup
	qarith
	qrange
	qelliptic
	qmemory
	qnnf
	qposext
	qposint
	sigma1
	sigma2
	sigma3
	sigma4
	id1
	id2
	id3
	id4
	table1
	table2
	table3
	table4
	lfirst
	llast
	wl
	wr
	wo
	w4
	zperm
	inverses
	readcounts
	readtags
	wls
	wrs
	wos
	w4s
	zperms
)

func (v *verifier) relations(p [41]V, t *transcript, domain V) V {
	a, m, s := v.Add, v.Mul, v.Sub
	var e [29]V
	for i := range e {
		e[i] = 0
	}
	// Arithmetic (0,1).
	q := p[qarith]
	acc := m(s(q, 3), m(p[qm], p[wr], p[wl]), integer("0x183227397098d014dc2822db40c0ac2e9419f4243cdcb848a1f0fac9f8000000"))
	acc = a(acc, m(p[ql], p[wl]), m(p[qr], p[wr]), m(p[qo], p[wo]), m(p[q4], p[w4]), p[qc])
	e[0] = m(a(acc, m(s(q, 1), p[w4s])), q, domain)
	e[1] = m(a(s(a(p[wl], p[w4]), p[wls]), p[qm]), s(q, 2), s(q, 1), q, domain)
	// Permutation (2,3,4), including z_perm initialization.
	num, den := V(1), V(1)
	for i, w := range []int{wl, wr, wo, w4} {
		num = m(num, a(p[w], m(p[id1+i], t.beta), t.gamma))
		den = m(den, a(p[w], m(p[sigma1+i], t.beta), t.gamma))
	}
	e[2] = m(s(m(a(p[zperm], p[lfirst]), num), m(a(p[zperms], m(p[llast], t.delta)), den)), domain)
	e[3] = m(p[llast], p[zperms], domain)
	e[4] = m(p[lfirst], p[zperm], domain)
	// Log-derivative lookup (5,6,7). Relation6 deliberately has no domain factor.
	b2, b3 := m(t.beta, t.beta), m(t.beta, t.beta, t.beta)
	table := a(p[table1], t.gamma, m(p[table2], t.beta), m(p[table3], b2), m(p[table4], b3))
	lookup := a(p[wl], t.gamma, m(p[qr], p[wls]), m(a(p[wr], m(p[qm], p[wrs])), t.beta), m(a(p[wo], m(p[qc], p[wos])), b2), m(p[qo], b3))
	exists := s(a(p[readtags], p[qlookup]), m(p[readtags], p[qlookup]))
	e[5] = m(s(m(lookup, table, p[inverses]), exists), domain)
	e[6] = s(m(p[qlookup], p[inverses], table), m(p[readcounts], p[inverses], lookup))
	e[7] = m(s(m(p[readtags], p[readtags]), p[readtags]), domain)
	// Delta range (8..11).
	for i, d := range []V{s(p[wr], p[wl]), s(p[wo], p[wr]), s(p[w4], p[wo]), s(p[wls], p[w4])} {
		e[8+i] = m(d, s(d, 1), s(d, 2), s(d, 3), p[qrange], domain)
	}
	// Grumpkin elliptic addition/doubling (12,13).
	x1, y1, x2, y2, x3, y3 := p[wr], p[wo], p[wls], p[w4s], p[wrs], p[wos]
	sign, dbl := p[ql], p[qm]
	dx, y1sq := s(x2, x1), m(y1, y1)
	xadd := a(s(m(a(x3, x2, x1), dx, dx), m(y2, y2), y1sq), m(2, y1, y2, sign))
	yadd := a(m(a(y1, y3), dx), m(s(x3, x1), s(m(y2, sign), y1)))
	e[12] = m(xadd, domain, p[qelliptic], s(1, dbl))
	e[13] = m(yadd, domain, p[qelliptic], s(1, dbl))
	xdouble := s(m(a(x3, x1, x1), 4, y1sq), m(a(y1sq, 17), x1, 9))
	ydouble := s(m(3, x1, x1, s(x1, x3)), m(2, y1, a(y1, y3)))
	e[12] = a(e[12], m(xdouble, domain, p[qelliptic], dbl))
	e[13] = a(e[13], m(ydouble, domain, p[qelliptic], dbl))
	// RAM/ROM (14..19).
	eta2, eta3 := m(t.eta, t.eta), m(t.eta, t.eta, t.eta)
	partialRecord := a(m(p[wo], eta3), m(p[wr], eta2), m(p[wl], t.eta), p[qc])
	record := s(partialRecord, p[w4])
	indexDelta, recordDelta := s(p[wls], p[wl]), s(p[w4s], p[w4])
	monotonic := m(indexDelta, s(indexDelta, 1))
	adjacent := m(s(1, indexDelta), recordDelta)
	romSelect := m(p[ql], p[qr])
	memScale := m(p[qmemory], domain)
	e[15] = m(adjacent, romSelect, memScale)
	e[16] = m(monotonic, romSelect, memScale)
	rom := m(record, romSelect)
	access := s(p[w4], partialRecord)
	accessCheck := m(access, s(access, 1))
	nextAccess := s(p[w4s], a(m(p[wos], eta3), m(p[wrs], eta2), m(p[wls], t.eta)))
	valueDelta := s(p[wos], p[wo])
	e[17] = m(s(1, indexDelta), valueDelta, s(1, nextAccess), p[qo], memScale)
	e[18] = m(monotonic, p[qo], memScale)
	e[19] = m(s(m(nextAccess, nextAccess), nextAccess), p[qo], memScale)
	ram := m(accessCheck, p[qo])
	timestamp := s(m(s(1, indexDelta), s(p[wrs], p[wr])), p[wo])
	e[14] = m(a(rom, m(timestamp, p[q4], p[ql]), m(record, p[qm], p[ql]), ram), memScale)
	// Non-native field relation (20), not silently omitted for a zero selector.
	limb := integer("0x100000000000000000") // 2^68
	sublimb := 1 << 14
	product := a(m(p[wl], p[wrs]), m(p[wls], p[wr]))
	gate2 := m(a(s(m(s(a(m(p[wl], p[w4]), m(p[wr], p[wo])), p[wos]), limb), p[w4s]), product), p[q4])
	product = a(m(product, limb), m(p[wls], p[wrs]))
	gate1 := m(s(product, a(p[wo], p[w4])), p[qo])
	gate3 := m(s(a(product, p[w4]), a(p[wos], p[w4s])), p[qm])
	nnf := m(a(gate1, gate2, gate3), p[qr])
	limb1 := a(m(p[wrs], sublimb), p[wls])
	for _, x := range []V{p[wo], p[wr], p[wl]} {
		limb1 = a(m(limb1, sublimb), x)
	}
	limb1 = m(s(limb1, p[w4]), p[q4])
	limb2 := a(m(p[wos], sublimb), p[wrs])
	for _, x := range []V{p[wls], p[w4], p[wo]} {
		limb2 = a(m(limb2, sublimb), x)
	}
	limb2 = m(s(limb2, p[w4s]), p[qm])
	e[20] = m(a(nnf, m(a(limb1, limb2), p[qo])), p[qnnf], domain)
	// Poseidon2 external round (21..24).
	fifth := func(x V) V { xx := m(x, x); return m(xx, xx, x) }
	u1, u2, u3, u4 := fifth(a(p[wl], p[ql])), fifth(a(p[wr], p[qr])), fifth(a(p[wo], p[qo])), fifth(a(p[w4], p[q4]))
	t0, t1 := a(u1, u2), a(u3, u4)
	t2, t3 := a(u2, u2, t1), a(u4, u4, t0)
	v4, v2 := a(m(4, t1), t3), a(m(4, t0), t2)
	for i, x := range []V{a(t3, v2), v2, a(t2, v4), v4} {
		e[21+i] = m(p[qposext], domain, s(x, p[wls+i]))
	}
	// Poseidon2 internal round (25..28).
	diagonal := []string{
		"0x10dc6e9c006ea38b04b1e03b4bd9490c0d03f98929ca1d7fb56821fd19d3b6e7",
		"0x0c28145b6a44df3e0149b3d0a30b3bb599df9756d4dd9b84a86b38cfb45a740b",
		"0x00544b8338791518b2c7645a50392798b21f75bb60e3596170067d00141cac15",
		"0x222c01175718386f2e2e82eb122789e352e105a3b8fa852613bc534433ee428b",
	}
	us := []V{fifth(a(p[wl], p[ql])), p[wr], p[wo], p[w4]}
	sum := a(us[0], us[1], us[2], us[3])
	for i, x := range us {
		e[25+i] = m(p[qposint], domain, s(a(m(x, integer(diagonal[i])), sum), p[wls+i]))
	}
	out := e[0]
	for i := 1; i < len(e); i++ {
		out = a(out, m(e[i], t.alpha[i-1]))
	}
	return out
}
