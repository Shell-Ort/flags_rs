use std::{
    cmp::Ordering::Equal,
    collections::BTreeMap,
    env::args,
    fmt::Debug,
    str::FromStr,
    sync::{Mutex, OnceLock},
};

static FLAGS: OnceLock<Mutex<BTreeMap<String, String>>> = OnceLock::new();

/// Function to be called
pub fn flag<T>(flag: &'static str, default_value: &T, description: &'static str) -> (T, bool)
where
    T: Clone + FromStr + ToString,
    <T as FromStr>::Err: Debug,
{
    let mut mp = parse_arguments().lock().unwrap();
    let flag = flag.to_string();

    // To know which ones we already visited, we save and = with the description + default value
    let (ret, exit) = if let Some(val) = mp.insert(flag, String::from("=") + description + "=" + default_value.to_string().as_str()) {
        (val.parse::<T>().unwrap(), true)
    } else {
        (default_value.clone(), false)
    };

    (ret, exit)
}

pub fn parse_flags() {
    let mp = parse_arguments().lock().unwrap();
    let bad_ones: Vec<_> =  mp.iter().filter(|(_,val)| !val.starts_with("=")).collect();
    if bad_ones.len() != 0 {
        let sal = bad_ones.into_iter().map(|(_,val)| val).cloned().collect::<Vec<_>>().join(", ") + ".";
        panic!("Unknown flags: \n{sal}");
    }
}

/// Function that will return de Binary Tree Map
/// Binary Tree and not HashMap because Hash is O(1) with a really big One
fn parse_arguments() -> &'static Mutex<BTreeMap<String, String>> {
    FLAGS.get_or_init(|| {
        let mut mp: BTreeMap<String, String> = BTreeMap::new();

        for arg in args() {
            if arg.starts_with("-") {
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
        }

        Mutex::new(mp)
    })
}

/// Is a separated function only for the tests
pub fn parse_pair(arg: &String) -> Option<(String, String)> {
    let (key, value) = match arg.find("=") {
        // In case -...=...
        Some(idx) => {
            if arg.chars().filter(|e| *e == '=').count() > 1 {
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
                arg.chars().skip(idx + 2).collect(),
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
    // use super::*;

    // #[test]
    // fn it_works() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }
}
