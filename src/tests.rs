#[cfg(test)]
mod test {
    use crate::crypt::*;
    #[test]
    fn enc_dec_test() {
        let key = vec![109,231,78,32,41,22,170,10];
        let msg = "Hi this is a random message to be encrypted!";
        let cipher = encrypt(&key, msg);
        assert_eq!(msg, &decrypt(&key, &cipher)[..]);
    }

    #[test]
    fn com_key_test() {
        use dh::*;
        let n = 2315981;
        let g = 772197;
        let kp1 = gen_key_pair(n, g);
        let kp2 = gen_key_pair(n, g);

        let s1 = gen_common_key(n, kp1.private, kp2.public);
        let s2 = gen_common_key(n, kp2.private, kp1.public);

        assert_eq!(s1, s2);
    }
}
