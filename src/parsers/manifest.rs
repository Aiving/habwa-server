use std::collections::BTreeMap;

pub fn parse(data: String) -> BTreeMap<String, String> {
    data.split('\n')
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| l.split(':').map(|s| s.trim()))
        .fold(BTreeMap::new(), |mut a, b| {
            let key = b.clone().max().unwrap().to_string();
            let value = b.clone().min().unwrap().to_string();

            if a.get(&key).is_none() {
                a.insert(key, value);
            }

            a
        })
}
