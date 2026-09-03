use lazy_static::lazy_static;
use spin::Mutex;

const MAX_SIGNALS: usize = 32;

// a signal handler is a plain function pointer
pub type SignalHandler = fn();

pub struct SignalTable {
    handlers: [Option<SignalHandler>; MAX_SIGNALS],
    // pending queue: signals scheduled but not yet delivered
    pending: [bool; MAX_SIGNALS],
}

impl SignalTable {
    pub const fn new() -> SignalTable {
        SignalTable {
            handlers: [None; MAX_SIGNALS],
            pending: [false; MAX_SIGNALS],
        }
    }

    // register a callback for signal `sig`
    pub fn register(&mut self, sig: usize, handler: SignalHandler) {
        if sig < MAX_SIGNALS {
            self.handlers[sig] = Some(handler);
        }
    }

    // raise a signal immediately — runs its handler now
    pub fn raise(&self, sig: usize) {
        if sig < MAX_SIGNALS {
            if let Some(handler) = self.handlers[sig] {
                handler();
            }
        }
    }

    // schedule a signal — marks it pending, delivered later by the main loop
    pub fn schedule(&mut self, sig: usize) {
        if sig < MAX_SIGNALS {
            self.pending[sig] = true;
        }
    }

    // returns the handlers to run; caller runs them AFTER releasing the lock
    pub fn take_pending(&mut self) -> [Option<SignalHandler>; MAX_SIGNALS] {
        let mut to_run: [Option<SignalHandler>; MAX_SIGNALS] = [None; MAX_SIGNALS];
        for sig in 0..MAX_SIGNALS {
            if self.pending[sig] {
                self.pending[sig] = false;
                to_run[sig] = self.handlers[sig];
            }
        }
        to_run
    }
}

lazy_static! {
    pub static ref SIGNALS: Mutex<SignalTable> = Mutex::new(SignalTable::new());
}