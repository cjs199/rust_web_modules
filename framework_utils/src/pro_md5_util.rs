


pub fn to_md5_str<T: AsRef<[u8]>>(data: T) -> String {
    let compute: md5::Digest = md5::compute(data);
    let compute_str = format!("{:x}", compute);
    compute_str
}