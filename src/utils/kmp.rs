pub struct Kmp {}

impl Kmp {
    /// 查找目标字节切片在数据字节切片中首次出现的位置
    pub fn find_first_target(data: &[u8], target: &[u8]) -> Option<usize> {
        if data.is_empty() || target.is_empty() {
            return None;
        }

        let combined = Self::combine_data_and_target(target, data);
        let lps = Self::compute_lps(&combined);

        let data_length = data.len();
        let target_length = target.len();

        for i in (target_length + 1)..=(data_length + target_length) {
            if lps[i] == target_length {
                return Some(i - 2 * target_length);
            }
        }

        None
    }

    /// 查找目标字节切片在数据字节切片中所有出现的位置
    pub fn find_all_targets(data: &[u8], target: &[u8]) -> Vec<usize> {
        if data.is_empty() || target.is_empty() {
            return vec![];
        }

        let combined = Self::combine_data_and_target(target, data);
        let lps = Self::compute_lps(&combined);

        let data_length = data.len();
        let target_length = target.len();

        let mut targets = vec![];
        for i in (target_length + 1)..=(data_length + target_length) {
            if lps[i] == target_length {
                targets.push(i - 2 * target_length);
            }
        }

        targets
    }

    /// 合并目标字节切片和数据字节切片，并添加分隔符
    fn combine_data_and_target(target: &[u8], data: &[u8]) -> Vec<u8> {
        let mut combined = Vec::with_capacity(target.len() + data.len() + 1);
        combined.extend_from_slice(target);
        combined.push(b'#');
        combined.extend_from_slice(data);
        combined
    }

    /// 计算给定字节切片的最长公共前后缀数组
    fn compute_lps(data: &[u8]) -> Vec<usize> {
        let n = data.len();
        let mut lps = vec![0; n];
        let mut prefix_end = 0;

        for i in 1..n {
            while prefix_end > 0 && data[i]!= data[prefix_end] {
                prefix_end = lps[prefix_end - 1];
            }
            if data[i] == data[prefix_end] {
                prefix_end += 1;
            }
            lps[i] = prefix_end;
        }

        lps
    }
}

#[cfg(test)]
mod tests {
    use super::Kmp;

    #[test]
    fn test_find_first_target() {
        let data = b"hello world";
        let target = b"world";
        let result = Kmp::find_first_target(data, target);
        assert_eq!(result, Some(6));
    }

    #[test]
    fn test_find_all_targets() {
        let data = b"ababab";
        let target = b"ab";
        let result = Kmp::find_all_targets(data, target);
        assert_eq!(result, vec![0, 2, 4]);
    }

    #[test]
    fn test_find_no_target() {
        let data = b"abcdef";
        let target = b"xyz";
        let result = Kmp::find_first_target(data, target);
        assert_eq!(result, None);
    }
}