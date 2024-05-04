use std::thread;
use std::time::Duration;

pub trait Registration {
    fn register(&self);
    fn deregister(&self);
}

pub trait Ooze {
    fn run(&self) -> thread::JoinHandle<()>;
}

pub struct Muck {
    pub config: MuckConfig,
    pub registration: Box<dyn Registration>,
    pub oozes: Vec<Box<dyn Ooze>>,
}

pub struct MuckConfig {
    pub name: String,
}

impl Muck {
    pub fn start(&self) {
        println!("Muck {} started successfully!", self.config.name);
        &self.registration.register();
        for ooze in &self.oozes {
            ooze.run();
        }

        loop {
            println!("Main thread is still running...");
            thread::sleep(Duration::from_secs(5));
        }
    }
}

// pub trait Registration {
//     fn register(&self, name: &str);
// }

// trait  {e
//     fn report_health(&self);
// }

// impl ReportHealth for Muck {
//     fn report_health(&self) {
//         log::info!("Muck {} started succesfully!", self.config.name)
//     }
// }
//
