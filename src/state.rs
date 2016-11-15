//! Pushdown automaton for state machine.

use super::Dobro;
type Context = Dobro;

/// Transition functions for the automaton.
pub enum Trans {
    /// No transition
    None,

    /// Removes the current state on the top of the stack and resumes
    /// the one below (stop if there's none).
    Pop,

    /// Pauses the current state on top of the stack and puts
    /// a new one on top of the stack.
    Push(Box<State>),

    /// Replaces the current state on top of the stack with a
    /// new one.
    Replace(Box<State>),

    /// Remotes all the states and quits.
    Quit,
}

/// A trait for types that can be used by the [Automaton](struct.Automaton.html).
pub trait State {
    /// Executed when the state is placed on top of the stack.
    fn start(&mut self, _ctx: &mut Context) {}

    /// Executed when the state is being removed from the top of the stack.
    fn stop(&mut self, _ctx: &mut Context) {}

    /// Executed when a new state is placed on top of this one.
    fn pause(&mut self, _ctx: &mut Context) {}

    /// Executed when this state becomes active once again.
    fn resume(&mut self, _ctx: &mut Context) {}

    /// Executed every cycle of the main loop.
    fn update(&mut self, _ctx: &mut Context) -> Trans { Trans::None }
}

/// Pushdown automaton.
pub struct Automaton {
    /// Set to true whenever the automaton is active and running.
    running: bool,

    /// Stack of boxed states. They are bosed so our automaton
    /// is the sole owner of the data.
    ///
    /// Also, the [State](trait.State.html) is not sized, so we need
    /// to used either Trait objects or pointers.
    state_stack: Vec<Box<State>>,
}

impl Automaton {
    /// Creates a new automaton.
    pub fn new<T>(initial_state: T) -> Self where T: State + 'static {
        Automaton {
            running: false,
            state_stack: vec![Box::new(initial_state)],
        }
    }

    /// Checks whether the automaton is running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Initializes the automaton.
    /// # Panics
    /// Panics if no states are on the stack.
    pub fn start(&mut self, ctx: &mut Context) {
        if !self.running {
            let state = self.state_stack.last_mut().unwrap();
            state.start(ctx);
            self.running = true;
        }
    }

    /// Updates the state on top of the stack.
    pub fn update(&mut self, ctx: &mut Context) {
        if self.running {
            let trans = match self.state_stack.last_mut() {
                Some(state) => state.update(ctx),
                None => Trans::None,
            };
            self.transition(trans, ctx);
        }
    }

    /// Process a transition.
    fn transition(&mut self, trans: Trans, ctx: &mut Context) {
        if self.running {
            match trans {
                Trans::None => (),
                Trans::Pop => self.pop(ctx),
                Trans::Push(state) => self.push(state, ctx),
                Trans::Replace(state) => self.replace(state, ctx),
                Trans::Quit => self.quit(ctx),
            }
        }
    }

    /// Pops state from the stack and resumes any state
    /// still on top.
    fn pop(&mut self, ctx: &mut Context) {
        if self.running {
            if let Some(mut state) = self.state_stack.pop() {
                state.stop(ctx);
            }
            if let Some(mut state) = self.state_stack.last_mut() {
                state.resume(ctx);
            }
            else {
                self.running = false;
            }
        }
    }

    /// Pushes a new state on top of the stack and pauses any state
    /// that was on top.
    fn push(&mut self, state: Box<State>, ctx: &mut Context) {
        if self.running {
            if let Some(mut state) = self.state_stack.last_mut() {
                state.pause(ctx);
            }
            self.state_stack.push(state);
            let state = self.state_stack.last_mut().unwrap();
            state.start(ctx);
        }
    }

    /// Replaces (if any) state on top of the stack.
    fn replace(&mut self, state: Box<State>, ctx: &mut Context) {
        if self.running {
            if let Some(mut state) = self.state_stack.pop() {
                state.stop(ctx);
            }
            self.state_stack.push(state);
            let state = self.state_stack.last_mut().unwrap();
            state.start(ctx);
        }
    }

    /// Quits the automaton.
    fn quit(&mut self, ctx: &mut Context) {
        if self.running {
            while let Some(mut state) = self.state_stack.pop() {
                state.stop(ctx);
            }
            self.running = false;
        }
    }
}
