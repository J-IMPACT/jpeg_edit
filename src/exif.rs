use little_exif::rational::uR64;

pub fn ur64_to_f64(u: &uR64) -> f64 {
    u.nominator as f64 / u.denominator as f64
}

pub fn dms_to_f64(v: &[uR64]) -> Option<f64> {
    if v.len() != 3 { return None; }
    Some(ur64_to_f64(&v[0]) + ur64_to_f64(&v[1]) / 60.0 + ur64_to_f64(&v[2]) / 3600.0)
}

pub fn f64_to_dms(value: f64) -> Option<Vec<uR64>> {
    if value < 0.0 { return None; }
    let (mut v, mut value) = (Vec::with_capacity(3), value);
    for i in 0..3 {
        v.push(value as u32);
        value = (value - v[i] as f64) * 60.0;
    }
    Some(vec![
        uR64 { nominator: v[0], denominator: 1 },
        uR64 { nominator: v[1], denominator: 1 },
        uR64 { nominator: v[2] * 100, denominator: 100 }
    ])
}

pub fn dir_to_f64(s: &str) -> Option<f64> {
    match s {
        "N" | "E" => Some(1.0),
        "S" | "W" => Some(-1.0),
        _ => None,
    }
}

pub fn latlng_to_exif(lat: f64, lng: f64) -> (String, Vec<uR64>, String, Vec<uR64>) {
    (
        if lat >= 0.0 { "N".to_string() } else { "S".to_string() },
        f64_to_dms(lat.abs()).unwrap(),
        if lng >= 0.0 { "E".to_string() } else { "W".to_string() },
        f64_to_dms(lng.abs()).unwrap(),
    )
}