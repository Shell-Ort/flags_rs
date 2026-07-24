use std::{
    any::type_name,
    cmp::Ordering::Equal,
    collections::BTreeMap,
    env::args,
    fmt::Debug,
    process::exit,
    str::FromStr,
    sync::{Mutex, OnceLock},
};

static FLAGS: OnceLock<Mutex<BTreeMap<String, String>>> = OnceLock::new();

/// Access to the flags. Only the first time is called all the flags are parsed, it is keep in memory for future calls.
///
/// It must be called only once per flag, more times will result in a Err
pub fn flag<T>(
    flag: &'static str,
    default_value: &T,
    description: &'static str,
) -> Result<(T, bool), String>
where
    T: Clone + FromStr + ToString,
    <T as FromStr>::Err: Debug,
{
    let mut mp = parse_arguments().lock().unwrap();
    let flag = flag.to_string();

    // To know which ones we already visited, we save and = with the description + default value
    let (ret, exit) = if let Some(val) = mp.insert(
        flag.clone(),
        String::from("=")
            + description
            + "="
            + default_value.to_string().as_str()
            + "="
            + type_name::<T>(),
    ) {
        if val.starts_with("=") {
            return Err(format!("Duplicate flag in code: {flag}"));
        }

        // Case of: -flag
        let nval = if val.is_empty() && type_name::<T>() == type_name::<bool>() {
            "true".parse::<T>().unwrap()
        } else {
            if let Ok(n) = val.parse::<T>() {
                n
            } else {
                return Err(format!(
                    "Value {val} cannot be parsed as {}.",
                    type_name::<T>()
                ));
            }
        };
        (nval, true)
    } else {
        (default_value.clone(), false)
    };

    Ok((ret, exit))
}

/// Must be called always after all the values of the flags ahve been retrieved (it will panic if called otherwise)
///
/// If a flag is passed by args and is not retrieved, it will panic. It will print help if passed.
pub fn parse_flags() {
    let mp = parse_arguments().lock().unwrap();

    let print_help = || {
        println!("Flags help: ");
        for (key, value) in mp.iter().filter(|(_, val)| val.starts_with("=")) {
            let sl: Vec<_> = value.split("=").collect();
            let (desc, default, typ) = (sl[1], sl[2], sl[3]);
            println!("-{key} {typ}");
            println!("\t{desc} (default {default})");
        }
    };

    if mp.contains_key("-help") || mp.contains_key("h") {
        if mp.len() == 1 {
            print_help();
            exit(1);
        } else {
            panic!("--help && -h are only use alone.")
        }
    }

    let bad_ones: Vec<_> = mp.iter().filter(|(_, val)| !val.starts_with("=")).collect();
    if !bad_ones.is_empty() {
        let sal = bad_ones
            .into_iter()
            .map(|(_, val)| val)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
            + ".";
        println!("Unknown flags: \n{sal}");
        print_help();
        panic!();
    }
}

/// Function that will return de Binary Tree Map
/// Binary Tree and not HashMap because Hash is O(1) with a really big One
fn parse_arguments() -> &'static Mutex<BTreeMap<String, String>> {
    FLAGS.get_or_init(|| {
        let mut mp: BTreeMap<String, String> = BTreeMap::new();

        for arg in args() {
            let Some((key, value)) = parse_pair(&arg) else {
                continue;
            };

            // Check if already

            if let Some(val) = mp.insert(key.clone(), value.clone()) {
                match val.cmp(&value) {
                    Equal => {
                        eprintln!("Redefinition of the same key and value: -{key}={value}")
                    }
                    _ => panic!("Doble value for same key: -{key}=({val}|{value})"),
                }
            };
        }

        Mutex::new(mp)
    })
}

/// Is a separated function only for the tests
fn parse_pair(arg: &String) -> Option<(String, String)> {
    if !arg.starts_with("-") {
        return None;
    }
    let (key, value) = match arg.find("=") {
        // In case -...=...
        Some(idx) => {
            if arg.chars().filter(|e| e.eq(&'=')).count() > 1 {
                eprintln!("Error in arg {arg}, only one = is valid.");
                return None;
            }
            if idx == 1 {
                eprintln!("Empty argument name in {arg} is not valid.");
                return None;
            }
            if arg.ends_with("=") {
                eprintln!(
                    "In case with no value, you should use {},not {arg}.",
                    arg.chars().take(arg.len() - 1).collect::<String>()
                );
            }
            (
                arg.chars().skip(1).take(idx - 1).collect(),
                arg.chars().skip(idx + 1).collect(),
            )
        }
        None => {
            if arg.len() == 1 {
                eprint!("Single - non valid.");
                return None;
            }
            (arg.chars().skip(1).collect(), String::new())
        }
    };
    Some((key, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = parse_pair(&String::from("-prueba=10"));
        assert_eq!(result, Some((String::from("prueba"), String::from("10"))));
    }

    #[test]
    fn flags_t() {
        let par = flag::<i32>("prueba", &10, "aaaa");
        assert_eq!(par, Ok((10, false)));
        let par = flag::<i32>("prueba", &10, "aaaa");
        assert_eq!(par, Err(String::from("Duplicate flag in code: prueba")));
        let par = flag::<bool>("past", &false, "asda");
        assert_eq!(par, Ok((false, false)));
    }
}
