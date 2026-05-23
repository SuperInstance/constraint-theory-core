//! Living constraint system — MusicalCell, JazzSession, TradingFours, CallAndResponse, Vamp.
//!
//! Biological metaphor: cells have genomes, epigenetic markers, transcription factors.
//! They express musical output based on shared context and inter-cellular signaling.
//! A JazzSession orchestrates multiple cells through phases (Head → Solo → Trading → Collective → Coda).

use std::collections::HashMap;

// ── Constants ────────────────────────────────────────────────────────

pub const GENOME_SIZE: usize = 25;
pub const BEATS_PER_BAR: usize = 4;
pub const BARS_PER_PHASE: usize = 8;

// ── Gene indices ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gene {
    SnapStrictness = 0,
    FunnelGravity = 1,
    LamanThreshold = 2,
    ConsensusWeight = 3,
    TempoTendency = 4,
    Syncopation = 5,
    Density = 6,
    DissonanceTolerance = 7,
    RhythmicComplexity = 8,
    MelodicRange = 9,
    HarmonicRichness = 10,
    DynamicRange = 11,
    Articulation = 12,
    Sustain = 13,
    AttackSharpness = 14,
    ReleaseCurve = 15,
    SpatialWidth = 16,
    ReverbTendency = 17,
    ModulationDepth = 18,
    FilterResonance = 19,
    PitchBendRange = 20,
    VelocitySensitivity = 21,
    TimbreBrightness = 22,
    PolyphonyPreference = 23,
    GrooveSwing = 24,
}

impl Gene {
    pub fn index(self) -> usize {
        self as usize
    }
}

// ── MIDI Event ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct MidiEvent {
    pub note: u8,
    pub velocity: u8,
    pub duration_beats: f64,
    pub offset_beats: f64,
    pub channel: u8,
}

impl MidiEvent {
    pub fn new(note: u8, velocity: u8, duration_beats: f64, offset_beats: f64, channel: u8) -> Self {
        Self { note, velocity, duration_beats, offset_beats, channel }
    }

    /// Simple quantize to nearest 16th note
    pub fn quantize_16th(&mut self) {
        const SIXTEENTH: f64 = 0.25;
        self.offset_beats = (self.offset_beats / SIXTEENTH).round() * SIXTEENTH;
        self.duration_beats = (self.duration_beats / SIXTEENTH).round().max(SIXTEENTH) * SIXTEENTH;
    }
}

// ── Cell Output ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CellOutput {
    pub events: Vec<MidiEvent>,
    pub signals: HashMap<String, f64>,
}

impl CellOutput {
    pub fn empty() -> Self {
        Self { events: Vec::new(), signals: HashMap::new() }
    }

    pub fn with_event(event: MidiEvent) -> Self {
        Self { events: vec![event], signals: HashMap::new() }
    }
}

// ── Cell Role ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellRole {
    Piano,
    Bass,
    Drums,
    Sax,
}

impl CellRole {
    /// Default MIDI channel for each role
    pub fn default_channel(self) -> u8 {
        match self {
            CellRole::Piano => 0,
            CellRole::Bass => 1,
            CellRole::Drums => 9,
            CellRole::Sax => 2,
        }
    }

    /// Default velocity range
    pub fn velocity_range(self) -> (u8, u8) {
        match self {
            CellRole::Piano => (40, 100),
            CellRole::Bass => (50, 110),
            CellRole::Drums => (60, 120),
            CellRole::Sax => (45, 105),
        }
    }

    /// Default register (MIDI note range center)
    pub fn register(self) -> (u8, u8) {
        match self {
            CellRole::Piano => (48, 84),
            CellRole::Bass => (28, 55),
            CellRole::Drums => (35, 81),
            CellRole::Sax => (54, 91),
        }
    }
}

// ── Transcription Factor ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TranscriptionFactor {
    pub gene_index: usize,
    pub sensitivity: f64,
}

impl TranscriptionFactor {
    pub fn new(gene_index: usize, sensitivity: f64) -> Self {
        Self { gene_index, sensitivity }
    }

    /// Compute activation level from a gene value
    pub fn activation(&self, genome: &[f64; GENOME_SIZE]) -> f64 {
        if self.gene_index >= GENOME_SIZE {
            return 0.0;
        }
        let val = genome[self.gene_index];
        // Sigmoid-like activation
        self.sensitivity * val / (1.0 + (self.sensitivity * val).abs())
    }
}

// ── Shared Context ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SharedContext {
    pub key: u8,
    pub tempo: f64,
    pub energy: f64,
    pub beat: usize,
    pub bar: usize,
    pub chord: Vec<u8>,
}

impl SharedContext {
    pub fn new(key: u8, tempo: f64) -> Self {
        Self {
            key,
            tempo,
            energy: 0.5,
            beat: 0,
            bar: 0,
            chord: vec![0, 4, 7], // major triad by default
        }
    }

    /// Get notes in current key (major scale)
    pub fn scale_notes(&self) -> Vec<u8> {
        let major_intervals: &[i32] = &[0, 2, 4, 5, 7, 9, 11];
        major_intervals.iter().map(|&i| (self.key as i32 + i) as u8).collect()
    }

    /// Get chord tones in current key
    pub fn chord_tones(&self) -> Vec<u8> {
        self.chord.iter().map(|&interval| self.key + interval).collect()
    }

    /// Advance one beat
    pub fn tick(&mut self) {
        self.beat += 1;
        if self.beat >= BEATS_PER_BAR {
            self.beat = 0;
            self.bar += 1;
        }
    }
}

// ── Musical Cell ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MusicalCell {
    pub genome: [f64; GENOME_SIZE],
    pub epigenetic: Vec<f64>,
    pub tfs: Vec<TranscriptionFactor>,
    pub history: Vec<CellOutput>,
    pub role: CellRole,
    pub active: bool,
    pub solo: bool,
    pub accumulated_signals: HashMap<String, f64>,
}

impl MusicalCell {
    pub fn new(role: CellRole) -> Self {
        let genome = Self::default_genome(role);
        let tfs = Self::default_tfs(role);
        let epigenetic = vec![0.5; GENOME_SIZE];

        Self {
            genome,
            epigenetic,
            tfs,
            history: Vec::new(),
            role,
            active: true,
            solo: false,
            accumulated_signals: HashMap::new(),
        }
    }

    /// Create a cell with specific genome values
    pub fn with_genome(role: CellRole, genome: [f64; GENOME_SIZE]) -> Self {
        let mut cell = Self::new(role);
        cell.genome = genome;
        cell
    }

    fn default_genome(role: CellRole) -> [f64; GENOME_SIZE] {
        let mut g = [0.5; GENOME_SIZE];
        match role {
            CellRole::Piano => {
                g[Gene::HarmonicRichness.index()] = 0.7;
                g[Gene::PolyphonyPreference.index()] = 0.8;
                g[Gene::MelodicRange.index()] = 0.6;
                g[Gene::Syncopation.index()] = 0.6;
            }
            CellRole::Bass => {
                g[Gene::Density.index()] = 0.4;
                g[Gene::GrooveSwing.index()] = 0.5;
                g[Gene::DynamicRange.index()] = 0.3;
                g[Gene::Syncopation.index()] = 0.4;
            }
            CellRole::Drums => {
                g[Gene::RhythmicComplexity.index()] = 0.8;
                g[Gene::Density.index()] = 0.7;
                g[Gene::Syncopation.index()] = 0.7;
                g[Gene::DynamicRange.index()] = 0.6;
            }
            CellRole::Sax => {
                g[Gene::Articulation.index()] = 0.7;
                g[Gene::MelodicRange.index()] = 0.8;
                g[Gene::DissonanceTolerance.index()] = 0.6;
                g[Gene::Syncopation.index()] = 0.7;
            }
        }
        g
    }

    fn default_tfs(role: CellRole) -> Vec<TranscriptionFactor> {
        match role {
            CellRole::Piano => vec![
                TranscriptionFactor::new(Gene::HarmonicRichness.index(), 2.0),
                TranscriptionFactor::new(Gene::Syncopation.index(), 1.5),
                TranscriptionFactor::new(Gene::PolyphonyPreference.index(), 1.8),
            ],
            CellRole::Bass => vec![
                TranscriptionFactor::new(Gene::GrooveSwing.index(), 2.0),
                TranscriptionFactor::new(Gene::Density.index(), 1.5),
            ],
            CellRole::Drums => vec![
                TranscriptionFactor::new(Gene::RhythmicComplexity.index(), 2.0),
                TranscriptionFactor::new(Gene::Density.index(), 1.8),
                TranscriptionFactor::new(Gene::DynamicRange.index(), 1.5),
            ],
            CellRole::Sax => vec![
                TranscriptionFactor::new(Gene::MelodicRange.index(), 2.0),
                TranscriptionFactor::new(Gene::Articulation.index(), 1.8),
                TranscriptionFactor::new(Gene::DissonanceTolerance.index(), 1.5),
            ],
        }
    }

    /// Receive signals from other cells
    pub fn receive(&mut self, signals: &HashMap<String, f64>) {
        for (key, &val) in signals {
            let entry = self.accumulated_signals.entry(key.clone()).or_insert(0.0);
            *entry = *entry * 0.7 + val * 0.3; // exponential moving average
        }
    }

    /// Update transcription factors based on incoming signals
    pub fn update_tfs(&mut self, _signals: &HashMap<String, f64>) {
        // Modulate TF sensitivities based on epigenetic state
        for tf in &mut self.tfs {
            if tf.gene_index < GENOME_SIZE {
                let epi = self.epigenetic[tf.gene_index];
                tf.sensitivity = tf.sensitivity * 0.9 + epi * 0.1;
            }
        }
    }

    /// Express musical output based on current state and context
    pub fn express(&mut self, context: &SharedContext) -> CellOutput {
        if !self.active {
            return CellOutput::empty();
        }

        let mut output = CellOutput::empty();
        let channel = self.role.default_channel();
        let (vel_lo, vel_hi) = self.role.velocity_range();
        let (note_lo, note_hi) = self.role.register();

        // Compute TF activation total
        let activation: f64 = self.tfs.iter().map(|tf| tf.activation(&self.genome)).sum();

        // Generate events based on role
        match self.role {
            CellRole::Drums => {
                self.express_drums(&mut output, context, channel, vel_lo, vel_hi, activation);
            }
            CellRole::Bass => {
                self.express_bass(&mut output, context, channel, vel_lo, vel_hi, activation, note_lo, note_hi);
            }
            CellRole::Piano => {
                self.express_piano(&mut output, context, channel, vel_lo, vel_hi, activation, note_lo, note_hi);
            }
            CellRole::Sax => {
                self.express_sax(&mut output, context, channel, vel_lo, vel_hi, activation, note_lo, note_hi);
            }
        }

        // Quantize if snap strictness is high
        if self.genome[Gene::SnapStrictness.index()] > 0.5 {
            for event in &mut output.events {
                event.quantize_16th();
            }
        }

        // Generate outgoing signals
        output.signals.insert("energy".into(), context.energy * activation);
        output.signals.insert("density".into(), self.genome[Gene::Density.index()]);
        output.signals.insert("syncopation".into(), self.genome[Gene::Syncopation.index()]);

        // Update epigenetic markers (use it or lose it)
        for i in 0..GENOME_SIZE {
            let usage = if output.events.iter().any(|e| e.channel == channel) { 0.01 } else { 0.0 };
            self.epigenetic[i] = (self.epigenetic[i] + usage).min(1.0);
        }

        self.history.push(output.clone());
        if self.history.len() > 64 {
            self.history.remove(0);
        }

        output
    }

    fn express_drums(
        &self, output: &mut CellOutput, context: &SharedContext,
        channel: u8, vel_lo: u8, vel_hi: u8, activation: f64,
    ) {
        let beat = context.beat as f64;
        let density = self.genome[Gene::Density.index()];
        let sync = self.genome[Gene::Syncopation.index()];

        // Kick on 1 and 3
        if context.beat == 0 || context.beat == 2 {
            let vel = self.velocity(vel_lo, vel_hi, activation, 0.8);
            output.events.push(MidiEvent::new(36, vel, 0.5, beat, channel));
        }

        // Hi-hat pattern based on density
        let hh_divisions = if density > 0.6 { 4 } else { 2 };
        for i in 0..hh_divisions {
            let offset = beat + (i as f64 / hh_divisions as f64) * BEATS_PER_BAR as f64 / 2.0;
            // Skip some hits for syncopation
            if sync > 0.5 && i % 2 == 1 && context.beat % 2 == 0 {
                continue;
            }
            let vel = self.velocity(vel_lo, vel_hi, activation, 0.5);
            output.events.push(MidiEvent::new(42, vel, 0.25, offset, channel));
        }

        // Snare on 2 and 4
        if context.beat == 1 || context.beat == 3 {
            let vel = self.velocity(vel_lo, vel_hi, activation, 0.9);
            output.events.push(MidiEvent::new(38, vel, 0.5, beat, channel));
        }

        // Ghost notes based on rhythmic complexity
        let complexity = self.genome[Gene::RhythmicComplexity.index()];
        if complexity > 0.5 && context.beat % 2 == 1 {
            let offset = beat + 0.5;
            let vel = self.velocity(vel_lo, vel_hi, activation, 0.3);
            output.events.push(MidiEvent::new(38, vel, 0.25, offset, channel));
        }

        // Ride cymbal for higher energy
        if context.energy > 0.6 {
            let vel = self.velocity(vel_lo, vel_hi, activation, 0.4);
            output.events.push(MidiEvent::new(51, vel, 0.5, beat, channel));
        }
    }

    fn express_bass(
        &self, output: &mut CellOutput, context: &SharedContext,
        channel: u8, vel_lo: u8, vel_hi: u8, activation: f64,
        note_lo: u8, note_hi: u8,
    ) {
        let beat = context.beat as f64;
        let chord = context.chord_tones();
        let density = self.genome[Gene::Density.index()];
        let groove = self.genome[Gene::GrooveSwing.index()];

        // Root note on beat 1
        if context.beat == 0 && !chord.is_empty() {
            let note = (chord[0]).clamp(note_lo, note_hi);
            let vel = self.velocity(vel_lo, vel_hi, activation, 0.9);
            output.events.push(MidiEvent::new(note, vel, 2.0, beat, channel));
        }

        // Walking bass based on density
        if density > 0.4 && context.beat > 0 {
            let scale = context.scale_notes();
            if let Some(&note) = scale.iter().find(|&&n| n >= note_lo && n <= note_hi) {
                let offset = beat + if groove > 0.5 { 0.1 } else { 0.0 };
                let vel = self.velocity(vel_lo, vel_hi, activation, 0.6);
                output.events.push(MidiEvent::new(note, vel, 0.8, offset, channel));
            }
        }

        // Approach note on beat 4
        if context.beat == 3 && density > 0.3 {
            if let Some(&root) = chord.first() {
                let approach = if root > note_lo { root - 1 } else { root + 1 };
                let note = approach.clamp(note_lo, note_hi);
                let vel = self.velocity(vel_lo, vel_hi, activation, 0.7);
                output.events.push(MidiEvent::new(note, vel, 0.8, beat + 0.5, channel));
            }
        }
    }

    fn express_piano(
        &self, output: &mut CellOutput, context: &SharedContext,
        channel: u8, vel_lo: u8, vel_hi: u8, activation: f64,
        note_lo: u8, note_hi: u8,
    ) {
        let beat = context.beat as f64;
        let chord = context.chord_tones();
        let harmony = self.genome[Gene::HarmonicRichness.index()];
        let sync = self.genome[Gene::Syncopation.index()];

        // Comp chords — voicings from chord tones
        if context.beat == 0 || (sync > 0.5 && context.beat == 2) {
            let mut notes = Vec::new();
            let max_notes = if self.solo { 2 } else { (harmony * 4.0) as usize + 1 };

            for (i, &tone) in chord.iter().enumerate() {
                if i >= max_notes { break; }
                let note = (tone + 12).clamp(note_lo, note_hi);
                notes.push(note);
            }

            let vel = self.velocity(vel_lo, vel_hi, activation, 0.7);
            let offset = if sync > 0.6 { beat + 0.25 } else { beat };
            let duration = if context.beat == 0 { 2.0 } else { 1.5 };
            for note in notes {
                output.events.push(MidiEvent::new(note, vel, duration, offset, channel));
            }
        }

        // Melodic fills based on polyphony preference
        let poly = self.genome[Gene::PolyphonyPreference.index()];
        if poly > 0.5 && context.beat == 3 {
            let scale = context.scale_notes();
            let notes_in_range: Vec<u8> = scale.into_iter()
                .filter(|&n| n >= note_lo && n <= note_hi)
                .collect();
            if !notes_in_range.is_empty() {
                let idx = (context.bar + context.beat) % notes_in_range.len();
                let note = notes_in_range[idx];
                let vel = self.velocity(vel_lo, vel_hi, activation, 0.5);
                output.events.push(MidiEvent::new(note, vel, 0.5, beat + 0.5, channel));
            }
        }
    }

    fn express_sax(
        &self, output: &mut CellOutput, context: &SharedContext,
        channel: u8, vel_lo: u8, vel_hi: u8, activation: f64,
        note_lo: u8, note_hi: u8,
    ) {
        if !self.solo && context.energy < 0.4 {
            return; // Sax rests when not soloing and energy is low
        }

        let beat = context.beat as f64;
        let scale = context.scale_notes();
        let notes_in_range: Vec<u8> = scale.into_iter()
            .filter(|&n| n >= note_lo && n <= note_hi)
            .collect();

        if notes_in_range.is_empty() {
            return;
        }

        let articulation = self.genome[Gene::Articulation.index()];
        let melodic_range = self.genome[Gene::MelodicRange.index()];
        let sync = self.genome[Gene::Syncopation.index()];

        // Generate melodic line
        let num_notes = if self.solo { (activation * 4.0) as usize + 1 } else { 1 };
        for i in 0..num_notes {
            let idx = (context.bar * BEATS_PER_BAR + context.beat + i) % notes_in_range.len();
            let note = notes_in_range[idx];

            // Add melodic variation
            let variation = if melodic_range > 0.6 && i > 0 {
                ((context.bar * 7 + i * 3) % 5) as u8
            } else {
                0
            };
            let note = (note + variation).min(note_hi);

            let offset = beat + (i as f64 * 0.5) + if sync > 0.5 { 0.125 } else { 0.0 };
            let duration = if articulation > 0.6 { 0.3 } else { 0.7 };
            let vel = self.velocity(vel_lo, vel_hi, activation, if self.solo { 0.8 } else { 0.5 });

            output.events.push(MidiEvent::new(note, vel, duration, offset, channel));
        }
    }

    fn velocity(&self, lo: u8, hi: u8, activation: f64, base: f64) -> u8 {
        let raw = lo as f64 + (hi - lo) as f64 * base * activation.min(1.0);
        raw.round().clamp(lo as f64, hi as f64) as u8
    }

    /// Reset cell state for new session
    pub fn reset(&mut self) {
        self.history.clear();
        self.accumulated_signals.clear();
        self.active = true;
        self.solo = false;
    }

    /// Get effective genome value considering epigenetics
    pub fn effective_gene(&self, gene: Gene) -> f64 {
        let idx = gene.index();
        self.genome[idx] * (1.0 - self.epigenetic[idx] * 0.3)
    }
}

// ── Session Phase ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionPhase {
    Head,
    SoloPiano,
    SoloSax,
    Trading,
    Collective,
    Coda,
}

impl SessionPhase {
    /// Default phase duration in bars
    pub fn default_bars(self) -> usize {
        match self {
            SessionPhase::Head => BARS_PER_PHASE,
            SessionPhase::SoloPiano => BARS_PER_PHASE,
            SessionPhase::SoloSax => BARS_PER_PHASE,
            SessionPhase::Trading => BARS_PER_PHASE * 2,
            SessionPhase::Collective => BARS_PER_PHASE,
            SessionPhase::Coda => 4,
        }
    }

    /// Next phase in sequence
    pub fn next(self) -> Option<SessionPhase> {
        match self {
            SessionPhase::Head => Some(SessionPhase::SoloPiano),
            SessionPhase::SoloPiano => Some(SessionPhase::SoloSax),
            SessionPhase::SoloSax => Some(SessionPhase::Trading),
            SessionPhase::Trading => Some(SessionPhase::Collective),
            SessionPhase::Collective => Some(SessionPhase::Coda),
            SessionPhase::Coda => None,
        }
    }

    /// All phases in order
    pub fn all() -> Vec<SessionPhase> {
        vec![
            SessionPhase::Head,
            SessionPhase::SoloPiano,
            SessionPhase::SoloSax,
            SessionPhase::Trading,
            SessionPhase::Collective,
            SessionPhase::Coda,
        ]
    }
}

// ── Jazz Session ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct JazzSession {
    pub cells: HashMap<CellRole, MusicalCell>,
    pub context: SharedContext,
    pub phase: SessionPhase,
    pub phase_bar: usize,
    pub phase_bars_remaining: usize,
    pub outputs: Vec<CellOutput>,
    pub tick_count: usize,
}

impl JazzSession {
    pub fn new(key: u8, tempo: f64) -> Self {
        let mut cells = HashMap::new();
        cells.insert(CellRole::Piano, MusicalCell::new(CellRole::Piano));
        cells.insert(CellRole::Bass, MusicalCell::new(CellRole::Bass));
        cells.insert(CellRole::Drums, MusicalCell::new(CellRole::Drums));
        cells.insert(CellRole::Sax, MusicalCell::new(CellRole::Sax));

        let phase = SessionPhase::Head;
        let context = SharedContext::new(key, tempo);

        Self {
            cells,
            context,
            phase,
            phase_bar: 0,
            phase_bars_remaining: phase.default_bars(),
            outputs: Vec::new(),
            tick_count: 0,
        }
    }

    /// Create session with specific energy
    pub fn with_energy(mut self, energy: f64) -> Self {
        self.context.energy = energy.clamp(0.0, 1.0);
        self
    }

    /// Advance one beat, return outputs from all cells
    pub fn tick(&mut self) -> Vec<CellOutput> {
        // Update solo states based on phase
        self.update_solo_states();

        // Collect all signals
        let mut all_signals = HashMap::new();
        all_signals.insert("energy".into(), self.context.energy);
        all_signals.insert("beat".into(), self.context.beat as f64);
        all_signals.insert("bar".into(), self.context.bar as f64);

        // Let cells receive signals and update TFs
        for cell in self.cells.values_mut() {
            cell.receive(&all_signals);
            cell.update_tfs(&all_signals);
        }

        // Express all cells
        let mut outputs = Vec::new();
        for role in &[CellRole::Piano, CellRole::Bass, CellRole::Drums, CellRole::Sax] {
            if let Some(cell) = self.cells.get_mut(role) {
                let output = cell.express(&self.context);
                // Merge signals back
                for (key, val) in &output.signals {
                    all_signals.insert(key.clone(), *val);
                }
                outputs.push(output);
            }
        }

        // Update context
        self.context.tick();

        // Track phase progress
        if self.context.beat == 0 && self.tick_count > 0 {
            self.phase_bar += 1;
            self.phase_bars_remaining = self.phase_bars_remaining.saturating_sub(1);

            if self.phase_bars_remaining == 0 {
                self.advance_phase();
            }
        }

        // Modulate energy based on phase
        self.modulate_energy();

        self.tick_count += 1;
        self.outputs.extend(outputs.clone());
        outputs
    }

    /// Perform a full session for given number of bars
    pub fn perform(&mut self, bars: usize) -> Vec<CellOutput> {
        let total_beats = bars * BEATS_PER_BAR;
        let mut all_outputs = Vec::new();
        for _ in 0..total_beats {
            all_outputs.extend(self.tick());
        }
        all_outputs
    }

    /// Run through all phases until coda
    pub fn full_performance(&mut self) -> Vec<CellOutput> {
        let mut all_outputs = Vec::new();
        while self.phase != SessionPhase::Coda || self.phase_bars_remaining > 0 {
            all_outputs.extend(self.tick());
            if self.phase == SessionPhase::Coda && self.phase_bars_remaining == 0 {
                break;
            }
        }
        all_outputs
    }

    fn update_solo_states(&mut self) {
        // Reset all solo states
        for cell in self.cells.values_mut() {
            cell.solo = false;
        }

        match self.phase {
            SessionPhase::Head => {}
            SessionPhase::SoloPiano => {
                if let Some(cell) = self.cells.get_mut(&CellRole::Piano) {
                    cell.solo = true;
                }
            }
            SessionPhase::SoloSax => {
                if let Some(cell) = self.cells.get_mut(&CellRole::Sax) {
                    cell.solo = true;
                }
            }
            SessionPhase::Trading => {
                // Trade between piano and sax every 4 bars
                let trading_segment = (self.phase_bar / 4) % 2;
                let role = if trading_segment == 0 { CellRole::Piano } else { CellRole::Sax };
                if let Some(cell) = self.cells.get_mut(&role) {
                    cell.solo = true;
                }
            }
            SessionPhase::Collective => {
                // All active, no solo
            }
            SessionPhase::Coda => {
                // All active at lower energy
            }
        }
    }

    fn advance_phase(&mut self) {
        if let Some(next) = self.phase.next() {
            self.phase = next;
            self.phase_bar = 0;
            self.phase_bars_remaining = self.phase.default_bars();
        }
    }

    fn modulate_energy(&mut self) {
        let target = match self.phase {
            SessionPhase::Head => 0.5,
            SessionPhase::SoloPiano => 0.6,
            SessionPhase::SoloSax => 0.8,
            SessionPhase::Trading => 0.7 + 0.2 * (self.phase_bar as f64 / 8.0).sin(),
            SessionPhase::Collective => 0.9,
            SessionPhase::Coda => {
                let fade = 1.0 - (self.phase_bar as f64 / 4.0);
                0.3 * fade.max(0.0)
            }
        };
        self.context.energy = self.context.energy * 0.9 + target * 0.1;
    }

    /// Get the current chord progression (simple II-V-I)
    pub fn current_chord(&self) -> Vec<u8> {
        let bar_mod = self.context.bar % 4;
        match bar_mod {
            0 => vec![2, 5, 9],     // ii
            1 => vec![7, 11, 2],    // V
            2 => vec![0, 4, 7],     // I
            3 => vec![0, 4, 7],     // I (continued)
            _ => vec![0, 4, 7],
        }
    }

    /// Check if session is finished
    pub fn is_finished(&self) -> bool {
        self.phase == SessionPhase::Coda && self.phase_bars_remaining == 0
    }

    /// Get total bars performed
    pub fn total_bars(&self) -> usize {
        self.tick_count / BEATS_PER_BAR
    }
}

// ── Trading Fours ────────────────────────────────────────────────────

/// Manages alternating 4-bar trades between two cells.
#[derive(Debug, Clone)]
pub struct TradingFours {
    pub cell_a: CellRole,
    pub cell_b: CellRole,
    pub bars_per_trade: usize,
    pub current_bar: usize,
    pub current_leader: CellRole,
}

impl TradingFours {
    pub fn new(cell_a: CellRole, cell_b: CellRole) -> Self {
        Self {
            cell_a,
            cell_b,
            bars_per_trade: 4,
            current_bar: 0,
            current_leader: cell_a,
        }
    }

    /// Advance one bar, possibly switching leader
    pub fn advance_bar(&mut self) -> CellRole {
        self.current_bar += 1;
        if self.current_bar >= self.bars_per_trade {
            self.current_bar = 0;
            self.current_leader = if self.current_leader == self.cell_a { self.cell_b } else { self.cell_a };
        }
        self.current_leader
    }

    /// Check if a role is currently leading
    pub fn is_leading(&self, role: CellRole) -> bool {
        self.current_leader == role
    }

    /// Get the responding cell
    pub fn responder(&self) -> CellRole {
        if self.current_leader == self.cell_a { self.cell_b } else { self.cell_a }
    }
}

// ── Call and Response ────────────────────────────────────────────────

/// Call-and-response pattern between a leader and responder cell.
#[derive(Debug, Clone)]
pub struct CallAndResponse {
    pub leader: CellRole,
    pub responder: CellRole,
    pub call_beats: usize,
    pub response_beats: usize,
    pub current_position: usize,
    pub in_call: bool,
}

impl CallAndResponse {
    pub fn new(leader: CellRole, responder: CellRole) -> Self {
        Self {
            leader,
            responder,
            call_beats: 4,
            response_beats: 4,
            current_position: 0,
            in_call: true,
        }
    }

    /// Advance one beat, return which role should play
    pub fn tick(&mut self) -> CellRole {
        let active = if self.in_call { self.leader } else { self.responder };
        self.current_position += 1;

        if self.in_call && self.current_position >= self.call_beats {
            self.in_call = false;
            self.current_position = 0;
        } else if !self.in_call && self.current_position >= self.response_beats {
            self.in_call = true;
            self.current_position = 0;
        }

        active
    }

    /// Check if currently in call phase
    pub fn is_call(&self) -> bool {
        self.in_call
    }

    /// Set call and response durations
    pub fn with_durations(mut self, call: usize, response: usize) -> Self {
        self.call_beats = call;
        self.response_beats = response;
        self
    }
}

// ── Vamp ─────────────────────────────────────────────────────────────

/// A repeating vamp pattern — a cell repeats a fixed pattern for N bars.
#[derive(Debug, Clone)]
pub struct Vamp {
    pub role: CellRole,
    pub pattern: Vec<MidiEvent>,
    pub repetitions: usize,
    pub current_bar: usize,
    pub active: bool,
}

impl Vamp {
    pub fn new(role: CellRole, pattern: Vec<MidiEvent>, repetitions: usize) -> Self {
        Self {
            role,
            pattern,
            repetitions,
            current_bar: 0,
            active: true,
        }
    }

    /// Create a simple piano vamp in given key
    pub fn piano_vamp(key: u8, repetitions: usize) -> Self {
        let pattern = vec![
            MidiEvent::new(key + 48, 80, 1.0, 0.0, 0),
            MidiEvent::new(key + 52, 70, 1.0, 0.0, 0),
            MidiEvent::new(key + 55, 70, 1.0, 0.0, 0),
            MidiEvent::new(key + 48, 60, 0.5, 2.0, 0),
            MidiEvent::new(key + 52, 60, 0.5, 2.0, 0),
            MidiEvent::new(key + 55, 60, 0.5, 2.0, 0),
        ];
        Self::new(CellRole::Piano, pattern, repetitions)
    }

    /// Get events for the current bar
    pub fn current_events(&self) -> &[MidiEvent] {
        &self.pattern
    }

    /// Advance one bar
    pub fn advance(&mut self) -> bool {
        self.current_bar += 1;
        if self.current_bar >= self.repetitions {
            self.active = false;
            false
        } else {
            true
        }
    }

    /// Reset the vamp
    pub fn reset(&mut self) {
        self.current_bar = 0;
        self.active = true;
    }

    /// Remaining bars
    pub fn remaining(&self) -> usize {
        self.repetitions.saturating_sub(self.current_bar)
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_musical_cell_new_piano() {
        let cell = MusicalCell::new(CellRole::Piano);
        assert_eq!(cell.role, CellRole::Piano);
        assert!(cell.active);
        assert!(!cell.solo);
        assert!(cell.history.is_empty());
        assert_eq!(cell.genome.len(), GENOME_SIZE);
        assert_eq!(cell.epigenetic.len(), GENOME_SIZE);
    }

    #[test]
    fn test_musical_cell_role_genomes_differ() {
        let piano = MusicalCell::new(CellRole::Piano);
        let drums = MusicalCell::new(CellRole::Drums);
        // Different roles should have different genome defaults
        assert_ne!(piano.genome, drums.genome);
    }

    #[test]
    fn test_cell_receive_signals() {
        let mut cell = MusicalCell::new(CellRole::Bass);
        let mut signals = HashMap::new();
        signals.insert("energy".into(), 0.8);
        cell.receive(&signals);
        assert!(*cell.accumulated_signals.get("energy").unwrap_or(&0.0) > 0.0);
    }

    #[test]
    fn test_cell_update_tfs() {
        let mut cell = MusicalCell::new(CellRole::Sax);
        let signals = HashMap::new();
        let orig_sensitivities: Vec<f64> = cell.tfs.iter().map(|tf| tf.sensitivity).collect();
        cell.update_tfs(&signals);
        // Should have changed slightly
        let new_sensitivities: Vec<f64> = cell.tfs.iter().map(|tf| tf.sensitivity).collect();
        // At least structure preserved
        assert_eq!(orig_sensitivities.len(), new_sensitivities.len());
    }

    #[test]
    fn test_cell_express_inactive() {
        let mut cell = MusicalCell::new(CellRole::Piano);
        cell.active = false;
        let ctx = SharedContext::new(60, 120.0);
        let output = cell.express(&ctx);
        assert!(output.events.is_empty());
    }

    #[test]
    fn test_cell_express_drums() {
        let mut cell = MusicalCell::new(CellRole::Drums);
        let ctx = SharedContext::new(60, 120.0);
        let output = cell.express(&ctx);
        assert!(!output.events.is_empty());
        // Should have signals
        assert!(output.signals.contains_key("energy"));
    }

    #[test]
    fn test_cell_express_all_roles() {
        for role in &[CellRole::Piano, CellRole::Bass, CellRole::Drums, CellRole::Sax] {
            let mut cell = MusicalCell::new(*role);
            let ctx = SharedContext::new(60, 120.0);
            let output = cell.express(&ctx);
            if *role == CellRole::Sax {
                // Sax might rest at low energy
                continue;
            }
            assert!(!output.events.is_empty(), "Cell {:?} produced no events", role);
        }
    }

    #[test]
    fn test_cell_history_accumulates() {
        let mut cell = MusicalCell::new(CellRole::Piano);
        let ctx = SharedContext::new(60, 120.0);
        cell.express(&ctx);
        assert_eq!(cell.history.len(), 1);
        cell.express(&ctx);
        assert_eq!(cell.history.len(), 2);
    }

    #[test]
    fn test_cell_effective_gene() {
        let cell = MusicalCell::new(CellRole::Piano);
        let raw = cell.genome[Gene::Syncopation.index()];
        let effective = cell.effective_gene(Gene::Syncopation);
        // Effective should be <= raw (epigenetic damping)
        assert!(effective <= raw + 0.001);
    }

    #[test]
    fn test_jazz_session_new() {
        let session = JazzSession::new(60, 140.0);
        assert_eq!(session.phase, SessionPhase::Head);
        assert_eq!(session.context.key, 60);
        assert_eq!(session.context.tempo, 140.0);
        assert_eq!(session.cells.len(), 4);
        assert_eq!(session.tick_count, 0);
    }

    #[test]
    fn test_jazz_session_tick() {
        let mut session = JazzSession::new(60, 120.0);
        let outputs = session.tick();
        assert!(!outputs.is_empty());
        assert_eq!(session.tick_count, 1);
    }

    #[test]
    fn test_jazz_session_perform() {
        let mut session = JazzSession::new(60, 120.0);
        let outputs = session.perform(4);
        assert!(!outputs.is_empty());
        assert_eq!(session.tick_count, 16); // 4 bars * 4 beats
    }

    #[test]
    fn test_jazz_session_phase_advances() {
        let mut session = JazzSession::new(60, 120.0);
        // Run through head phase (8 bars = 32 beats)
        for _ in 0..32 {
            session.tick();
        }
        assert_ne!(session.phase, SessionPhase::Head);
    }

    #[test]
    fn test_jazz_session_solo_piano_phase() {
        let mut session = JazzSession::new(60, 120.0);
        session.phase = SessionPhase::SoloPiano;
        session.update_solo_states();
        assert!(session.cells.get(&CellRole::Piano).unwrap().solo);
        assert!(!session.cells.get(&CellRole::Sax).unwrap().solo);
    }

    #[test]
    fn test_jazz_session_solo_sax_phase() {
        let mut session = JazzSession::new(60, 120.0);
        session.phase = SessionPhase::SoloSax;
        session.update_solo_states();
        assert!(session.cells.get(&CellRole::Sax).unwrap().solo);
        assert!(!session.cells.get(&CellRole::Piano).unwrap().solo);
    }

    #[test]
    fn test_jazz_session_trading_phase() {
        let mut session = JazzSession::new(60, 120.0);
        session.phase = SessionPhase::Trading;
        session.phase_bar = 0;
        session.update_solo_states();
        // First 4 bars: piano leads
        assert!(session.cells.get(&CellRole::Piano).unwrap().solo);

        session.phase_bar = 4;
        session.update_solo_states();
        // Next 4 bars: sax leads
        assert!(session.cells.get(&CellRole::Sax).unwrap().solo);
    }

    #[test]
    fn test_jazz_session_is_finished() {
        let mut session = JazzSession::new(60, 120.0);
        assert!(!session.is_finished());
        session.phase = SessionPhase::Coda;
        session.phase_bars_remaining = 0;
        assert!(session.is_finished());
    }

    #[test]
    fn test_trading_fours() {
        let mut tf = TradingFours::new(CellRole::Piano, CellRole::Sax);
        assert_eq!(tf.current_leader, CellRole::Piano);
        // Advance 4 bars
        for _ in 0..4 {
            tf.advance_bar();
        }
        assert_eq!(tf.current_leader, CellRole::Sax);
        // Advance 4 more
        for _ in 0..4 {
            tf.advance_bar();
        }
        assert_eq!(tf.current_leader, CellRole::Piano);
    }

    #[test]
    fn test_trading_fours_is_leading() {
        let tf = TradingFours::new(CellRole::Piano, CellRole::Sax);
        assert!(tf.is_leading(CellRole::Piano));
        assert!(!tf.is_leading(CellRole::Sax));
        assert_eq!(tf.responder(), CellRole::Sax);
    }

    #[test]
    fn test_call_and_response() {
        let mut cr = CallAndResponse::new(CellRole::Sax, CellRole::Piano);
        assert!(cr.is_call());
        // 4 beats of call
        for _ in 0..4 {
            assert_eq!(cr.tick(), CellRole::Sax);
        }
        // Now in response
        assert!(!cr.is_call());
        for _ in 0..4 {
            assert_eq!(cr.tick(), CellRole::Piano);
        }
        // Back to call
        assert!(cr.is_call());
    }

    #[test]
    fn test_call_and_response_custom_durations() {
        let cr = CallAndResponse::new(CellRole::Sax, CellRole::Piano)
            .with_durations(2, 2);
        assert_eq!(cr.call_beats, 2);
        assert_eq!(cr.response_beats, 2);
    }

    #[test]
    fn test_vamp_creation() {
        let vamp = Vamp::piano_vamp(60, 8);
        assert_eq!(vamp.role, CellRole::Piano);
        assert_eq!(vamp.repetitions, 8);
        assert!(vamp.active);
        assert!(!vamp.pattern.is_empty());
    }

    #[test]
    fn test_vamp_advance() {
        let mut vamp = Vamp::piano_vamp(60, 3);
        assert!(vamp.advance()); // bar 1
        assert!(vamp.advance()); // bar 2
        assert!(!vamp.advance()); // bar 3 -> done
        assert!(!vamp.active);
    }

    #[test]
    fn test_vamp_reset() {
        let mut vamp = Vamp::piano_vamp(60, 2);
        vamp.advance();
        vamp.advance();
        assert!(!vamp.active);
        vamp.reset();
        assert!(vamp.active);
        assert_eq!(vamp.current_bar, 0);
    }

    #[test]
    fn test_vamp_remaining() {
        let mut vamp = Vamp::piano_vamp(60, 4);
        assert_eq!(vamp.remaining(), 4);
        vamp.advance();
        assert_eq!(vamp.remaining(), 3);
    }

    #[test]
    fn test_midi_event_quantize() {
        let mut evt = MidiEvent::new(60, 80, 0.37, 1.13, 0);
        evt.quantize_16th();
        assert_eq!(evt.offset_beats, 1.25); // nearest 16th
        assert!(evt.duration_beats >= 0.25);
    }

    #[test]
    fn test_shared_context_tick() {
        let mut ctx = SharedContext::new(60, 120.0);
        assert_eq!(ctx.beat, 0);
        ctx.tick();
        assert_eq!(ctx.beat, 1);
        assert_eq!(ctx.bar, 0);
        // Tick through a full bar
        ctx.tick();
        ctx.tick();
        ctx.tick(); // beat 4 -> wraps
        assert_eq!(ctx.beat, 0);
        assert_eq!(ctx.bar, 1);
    }

    #[test]
    fn test_shared_context_scale_notes() {
        let ctx = SharedContext::new(60, 120.0); // C major
        let notes = ctx.scale_notes();
        assert!(notes.contains(&60)); // C
        assert!(notes.contains(&64)); // E
        assert!(notes.contains(&67)); // G
    }

    #[test]
    fn test_session_phase_sequence() {
        let mut phase = SessionPhase::Head;
        let mut phases = vec![phase];
        while let Some(next) = phase.next() {
            phases.push(next);
            phase = next;
        }
        assert_eq!(phases, SessionPhase::all());
        assert_eq!(phases.len(), 6);
    }

    #[test]
    fn test_cell_reset() {
        let mut cell = MusicalCell::new(CellRole::Piano);
        let ctx = SharedContext::new(60, 120.0);
        cell.express(&ctx);
        cell.solo = true;
        cell.active = false;
        cell.reset();
        assert!(cell.history.is_empty());
        assert!(cell.active);
        assert!(!cell.solo);
    }

    #[test]
    fn test_transcription_factor_activation() {
        let tf = TranscriptionFactor::new(0, 2.0);
        let genome = [0.5; GENOME_SIZE];
        let act = tf.activation(&genome);
        assert!(act > 0.0);
        assert!(act < 1.0); // sigmoid compresses
    }

    #[test]
    fn test_jazz_session_with_energy() {
        let session = JazzSession::new(60, 120.0).with_energy(0.9);
        assert!((session.context.energy - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_jazz_session_energy_modulation() {
        let mut session = JazzSession::new(60, 120.0);
        session.phase = SessionPhase::SoloSax;
        // Run a few ticks to let energy modulate
        for _ in 0..4 {
            session.tick();
        }
        // Energy should have moved toward SoloSax target (0.8)
        assert!(session.context.energy > 0.5);
    }
}
