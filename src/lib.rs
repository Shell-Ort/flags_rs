use std::{
    any::type_name, cmp::Ordering::Equal, collections::{BTreeMap, HashSet}, env::args, fmt::Debug, process::exit, str::FromStr, sync::{Mutex, OnceLock},
};

struct Savings {
    mp: BTreeMap<String, String>,
    extras: Vec<String>,
}

static FLAGS: OnceLock<Mutex<Savings>> = OnceLock::new();

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
    let mut data = parse_arguments().lock().unwrap();
    let mp = &mut data.mp;
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

/// Must be called always after all the values of the flags.
///
/// Is to  print help if needed.
pub fn parse_flags() {
    let data = parse_arguments().lock().unwrap();
    let mp = &data.mp;

    if mp.contains_key("-help") || mp.contains_key("h") {
        if mp.len() == 1 {
            println!("Flags help: ");
            for (key, value) in mp.iter().filter(|(_, val)| val.starts_with("=")) {
                let sl: Vec<_> = value.split("=").collect();
                let (desc, default, typ) = (sl[1], sl[2], sl[3]);
                println!("-{key} {typ}");
                println!("\t{desc} (default {default})");
            }
        } else {
            eprintln!("--help && -h are only use alone.")
        }
        exit(1);
    }
}

/// Return a copy of the vector of the args non flags
pub fn non_flags() -> Vec<String> {
    parse_arguments().lock().unwrap().extras.clone()
}

// Function that will return de Binary Tree Map
// Binary Tree and not HashMap because Hash is O(1) with a really big One
fn parse_arguments() -> &'static Mutex<Savings> {
    FLAGS.get_or_init(|| {
        let mut mp: BTreeMap<String, String> = BTreeMap::new();
        let mut extras: HashSet<String> = HashSet::new();

        for arg in args().skip(1) {
            let (key,value) = match parse_pair(&arg) {
                Ok((k,v)) => (k,v),
                Err(BadFlag::NotFlag) => {
                    extras.insert(arg);
                    continue;
                },
                _ => continue,
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

        let extras: Vec<String> = extras.into_iter().collect();

        Mutex::new(Savings { mp, extras })
    })
}

#[derive(PartialEq, Eq, Debug)]
enum BadFlag {
    NotFlag,
    BadSyntax,
}

// Is a separated function only for the tests
fn parse_pair(arg: &String) -> Result<(String, String), BadFlag> {
    if !arg.starts_with("-") {
        return Err(BadFlag::NotFlag);
    }
    let (key, value) = match arg.find("=") {
        // In case -...=...
        Some(idx) => {
            if arg.chars().filter(|e| e.eq(&'=')).count() > 1 {
                eprintln!("Error in arg {arg}, only one = is valid.");
                return Err(BadFlag::BadSyntax);
            }
            if idx == 1 {
                eprintln!("Empty argument name in {arg} is not valid.");
                return Err(BadFlag::BadSyntax);
            }
            if arg.ends_with("=") {
                eprintln!(
                    "In case with no value, you should use {},not {arg}.",
                    arg.chars().take(arg.len() - 1).collect::<String>()
                );
                return Err(BadFlag::BadSyntax);
            }
            (
                arg.chars().skip(1).take(idx - 1).collect(),
                arg.chars().skip(idx + 1).collect(),
            )
        }
        None => {
            if arg.len() == 1 {
                eprint!("Single - non valid.");
                return Err(BadFlag::BadSyntax);
            }
            (arg.chars().skip(1).collect(), String::new())
        }
    };
    Ok((key, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = parse_pair(&String::from("-prueba=10"));
        assert_eq!(result, Ok((String::from("prueba"), String::from("10"))));
    }

    #[test]
    fn flags_t() {
        let par = flag::<i32>("prueba", &10, "aaaa");
        assert_eq!(par, Ok((10, false)));
        let par = flag::<i32>("prueba", &10, "aaaa");
        assert_eq!(par, Err(String::from("Duplicate flag in code: prueba")));
        let par = flag::<bool>("past", &false, "asda");
        assert_eq!(par, Ok((false, false)));
        let (name, _exists) =
            flag::<String>("name", &"world".to_string(), "Name to greet.").unwrap();
        assert_eq!(name, String::from("world"));
    }

    #[test]
    fn parse_pair_invalid() {
        assert_eq!(parse_pair(&String::from("hello")), Err(BadFlag::NotFlag));
        assert_eq!(parse_pair(&String::from("-=10")), Err(BadFlag::BadSyntax));
        assert_eq!(parse_pair(&String::from("-flag=")), Err(BadFlag::BadSyntax));
    }

    #[test]
    fn bool_flag_default() {
        let (verbose, exists) = flag::<bool>("verbose", &false, "ajdhakjds").unwrap();
        assert_eq!(verbose, false);
        assert!(!exists);
    }
}
