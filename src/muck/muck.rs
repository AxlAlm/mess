use crate::ooze;
use crate::registration;
use std::thread;
use std::time::Duration;

pub struct Muck {
    pub config: MuckConfig,
    pub registration: Box<dyn registration::Registration>,
    pub oozes: Vec<Box<dyn ooze::Ooze>>,
}

pub struct MuckConfig {
    pub name: String,
}

impl Muck {
    pub fn start(&self) {
        println!("Muck {} started successfully!", self.config.name);

        match &self.registration.register() {
            Ok(_) => println!("Service registered successfully."),
            Err(e) => panic!("Error registering service: {}", e),
        }

        for ooze in &self.oozes {
            match ooze.run() {
                Ok(_) => println!("Ooze started"),
                Err(e) => panic!("Error running Ooze: {}", e),
            }
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
