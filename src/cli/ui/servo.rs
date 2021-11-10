use std::{sync::mpsc::Sender, thread::spawn, time::Duration};

use crossterm::event::Event as CrossTermEvent;

use log::{info, warn};

use crate::{
    cli::{
        ui::utils::{Interval, Pulse},
        Event, Flags, Logger, TuiOpt, UpdateEvent,
    },
    Result,
};

pub(super) fn servo(tx: Sender<Event>, opt: &TuiOpt, flags: &Flags) -> Result<()> {
    macro_rules! run {
            ($block:block) => {
                Some(spawn(move || -> Result<()> {
                    $block
                    Ok(())
                }))
            }
        }

    macro_rules! watch {
            ($identifier:literal, $handle:ident) => {
                if let Some(ref handle) = $handle {
                    if !handle.is_running() {
                        let handle = $handle.take().unwrap();
                        match handle.join() {
                            Ok(res) => warn!(
                                "Background task `{}` has stopped running ({:?})",
                                $identifier, res
                            ),
                            Err(e) => warn!(
                                "Catastrophic failure: Background task `{}` has stopped running ({:?})",
                                $identifier, e
                            ),
                        }
                    }
                }
            };
        }

    Logger::new(tx.clone()).apply().unwrap();

    info!("Logger set");

    let key_tx = tx.clone();
    let traffic_tx = tx.clone();
    let log_tx = tx.clone();

    let clash = flags.connect_server_from_config()?;
    let req_clash = clash.clone();

    let mut key_handle = run!({
        loop {
            match crossterm::event::read() {
                Ok(CrossTermEvent::Key(event)) => key_tx.send(Event::from(event))?,
                Err(_) => {
                    key_tx.send(Event::Quit)?;
                    break;
                }
                _ => {}
            }
        }
    });

    let mut traffics = clash.get_traffic()?;
    #[allow(unreachable_code)]
    let mut traffic_handle = run!({
        loop {
            match traffics.next() {
                Some(Ok(traffic)) => {
                    traffic_tx.send(Event::Update(UpdateEvent::Traffic(traffic)))?
                }
                // Some(Ok(traffic)) => info!("{}", traffic),
                Some(Err(e)) => warn!("{:?}", e),
                None => warn!("No more traffic"),
            }
        }
    });

    let mut logs = clash.get_log()?;
    #[allow(unreachable_code)]
    let mut log_handle = run!({
        loop {
            match logs.next() {
                Some(Ok(log)) => log_tx.send(Event::Update(UpdateEvent::Log(log)))?,
                Some(Err(e)) => warn!("{:?}", e),
                None => warn!("No more traffic"),
            }
        }
    });

    #[allow(unreachable_code)]
    let mut req_handle = run!({
        let mut interval = Interval::every(Duration::from_millis(50));
        let mut connection_pulse = Pulse::new(10); // Every 500 ms
        let mut proxies_pulse = Pulse::new(40); // Every 2 s
        let mut version_pulse = Pulse::new(200); // Every 10 s

        let clash = req_clash;
        loop {
            if version_pulse.tick() {
                tx.send(Event::Update(UpdateEvent::Version(clash.get_version()?)))?;
            }
            if connection_pulse.tick() {
                tx.send(Event::Update(UpdateEvent::Connection(
                    clash.get_connections()?,
                )))?;
            }
            if proxies_pulse.tick() {
                tx.send(Event::Update(UpdateEvent::Proxies(clash.get_proxies()?)))?;
            }
            interval.tick();
        }
    });

    loop {
        watch!("key", key_handle);
        watch!("traffic", traffic_handle);
        watch!("log", log_handle);
        watch!("request", req_handle);
    }
}
