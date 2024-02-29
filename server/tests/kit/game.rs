use mafia_server::game::{phase::{PhaseState, PhaseType}, Game};

pub struct TestGame (*mut Game);

impl std::ops::Deref for TestGame {
    type Target = Game;

    fn deref(&self) -> &Self::Target {
        unsafe {&*self.0}
    }
}

impl std::ops::DerefMut for TestGame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {&mut *self.0}
    }
}

impl TestGame {
    pub fn new(game: &mut Game) -> Self {
        TestGame(game as *mut Game)
    }

    /// Advance the game naturally, passing through all the phases, until the given day and phase is met.
    /// ### Panics:
    /// * When the supplied phase doesn't always happen, like Judgement.
    /// * When the specified phase *cannot* happen, like Discussion 1.
    /// * When the specified day and phase is in the past.
    /// * If this would take the game to a day past the maximum day
    pub fn skip_to(&mut self, phase: PhaseType, day_number: u8) -> &PhaseState {
        // If the phase & day is in the past
        if self.day_number() > day_number || (self.day_number() == day_number && self.current_phase().phase() > phase) {
            panic!("Can't skip back in time! Tried to go to {:?} {}, but was already on {:?} {}, skip_to", phase, day_number, self.current_phase().phase(), self.day_number());
        }

        while self.day_number() != day_number || self.current_phase().phase() != phase {
            if self.day_number() == u8::MAX - 1 && self.current_phase().phase() == PhaseType::Night {
                panic!("Can't go above the maximum day, skip_to");
            }

            if self.day_number() > day_number {
                panic!("Phase {phase:?} {day_number} never occurs in the future!, skip_to");
            }

            self.next_phase();

            if self.day_number() > day_number || (self.day_number() == day_number && self.current_phase().phase() > phase) {
                panic!("That phase was skipped past. Maybe your test subjects need to vote someone? Tried to go to {:?} {}, but was already on {:?} {}, skip_to", phase, day_number, self.current_phase().phase(), self.day_number());
            }
        }

        self.current_phase()
    }
    pub fn next_phase(&mut self){
        let next_phase = PhaseState::end(self);
        self.start_phase(next_phase);
    }
}