use libcfhdb::pci::*;
use crate::config::*;
use std::fs;

fn get_pci_profiles_from_url() -> Result<Vec<CfhdbPciProfile>, std::io::Error> {
    let cached_db_path = std::path::Path::new("/var/cache/cfhdb/pci.json");
    let data = match reqwest::blocking::get(PCI_PROFILE_JSON_URL) {
        Ok(t) => {
            println!("[Info] Downloaded pci json from url");
            let cache = t.text().unwrap();
            let _ = std::fs::File::create(cached_db_path);
            let _ = fs::write(cached_db_path, &cache);
            cache
        },
        Err(_) => {
            println!("[Warn] Failed to download PCI Profile json from url, falling back to cache!");
            if cached_db_path.exists() {
                println!("[Info] PCI Profile Cache found!");
                std::fs::read_to_string(cached_db_path).unwrap()
            } else {
                println!("[Error] Could not find PCI Profile offline cache");
                return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "PCI Cache could not be found"))
            }
        }
    };
    let mut profiles_array = vec![];
    let res: serde_json::Value = serde_json::from_str(&data).expect("Unable to parse");
    if let serde_json::Value::Array(profiles) = &res["profiles"] {
        for profile in profiles {
            let codename = profile["codename"].as_str().unwrap_or_default().to_string();
            let i18n_desc = match profile[format!("i18n_desc[{}]", rust_i18n::locale().to_string())].as_str() {
                Some(t) => {
                    if !t.is_empty() {
                        t.to_string()
                    } else {
                        profile["i18n_desc"].as_str().unwrap_or_default().to_string()
                    }
                }
                None => {
                    profile["i18n_desc"].as_str().unwrap_or_default().to_string()
                }
            };
            let icon_name = profile["icon_name"]
                .as_str()
                .unwrap_or("package-x-generic")
                .to_string();
            let class_ids: Vec<String> = profile["class_ids"]
                .as_array()
                .expect("invalid_usb_profile_class_ids")
                .into_iter()
                .map(|x| x.as_str().unwrap_or_default().to_string())
                .collect();
            let vendor_ids: Vec<String> = profile["vendor_ids"]
                .as_array()
                .expect("invalid_usb_profile_class_ids")
                .into_iter()
                .map(|x| x.as_str().unwrap_or_default().to_string())
                .collect();
            let device_ids: Vec<String> = profile["device_ids"]
                .as_array()
                .expect("invalid_usb_profile_class_ids")
                .into_iter()
                .map(|x| x.as_str().unwrap_or_default().to_string())
                .collect();
            let blacklisted_class_ids: Vec<String> = profile["blacklisted_class_ids"]
                .as_array()
                .expect("invalid_usb_profile_class_ids")
                .into_iter()
                .map(|x| x.as_str().unwrap_or_default().to_string())
                .collect();
            let blacklisted_vendor_ids: Vec<String> = profile["blacklisted_vendor_ids"]
                .as_array()
                .expect("invalid_usb_profile_class_ids")
                .into_iter()
                .map(|x| x.as_str().unwrap_or_default().to_string())
                .collect();
            let blacklisted_device_ids: Vec<String> = profile["blacklisted_device_ids"]
                .as_array()
                .expect("invalid_usb_profile_class_ids")
                .into_iter()
                .map(|x| x.as_str().unwrap_or_default().to_string())
                .collect();
            let packages: Option<Vec<String>> = match profile["packages"].as_str() {
                Some(_) => None,
                None => Some(
                    profile["packages"]
                        .as_array()
                        .expect("invalid_usb_profile_class_ids")
                        .into_iter()
                        .map(|x| x.as_str().unwrap_or_default().to_string())
                        .collect(),
                ),
            };
            let install_script_value = profile["install_script"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let install_script = match install_script_value.as_str() {
                "Option::is_none" => None,
                _ => Some(install_script_value),
            };
            let remove_script_value = profile["remove_script"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let remove_script = match remove_script_value.as_str() {
                "Option::is_none" => None,
                _ => Some(remove_script_value),
            };
            let experimental = profile["experimental"].as_bool().unwrap_or_default();
            let removable = profile["removable"].as_bool().unwrap_or_default();
            let priority = profile["priority"].as_i64().unwrap_or_default();
            // Parse into the Struct
            let profile_struct = CfhdbPciProfile {
                codename,
                i18n_desc,
                icon_name,
                class_ids,
                vendor_ids,
                device_ids,
                blacklisted_class_ids,
                blacklisted_vendor_ids,
                blacklisted_device_ids,
                packages,
                install_script,
                remove_script,
                experimental,
                removable,
                priority: priority as i32,
            };
            profiles_array.push(profile_struct);
            profiles_array.sort_by_key(|x| x.priority);
        }
    }
    Ok(profiles_array)
}