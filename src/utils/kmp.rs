pub struct Kmp {}

impl Kmp {
    pub fn find_first_occurrence(data: &[u8], target: &[u8]) -> Option<usize> {
        if data.is_empty() || target.is_empty() {
            return None;
        }

        // Combine target and data with a separator
        let mut combined = Vec::new();
        combined.extend_from_slice(target); // Add the target
        combined.push(b'#'); // Add a separator
        combined.extend_from_slice(data); // Add the data

        let data_length = data.len(); // Size of the data
        let target_length = target.len(); // Size of the target

        let lps = Self::longest_prefix_suffix(&combined); // Get the prefix

        // Check for occurrences
        for i in (target_length + 1)..=(data_length + target_length) {
            if lps[i] == target_length {
                return Some(i - 2 * target_length); // Calculate start index of the occurrence
            }
        }

        None // not found
    }

    pub fn find_all_occurrences(data: &[u8], target: &[u8]) -> Vec<usize> {
        if data.is_empty() || target.is_empty() {
            return vec![];
        }

        // Combine target and data with a separator
        let mut combined = Vec::new();
        combined.extend_from_slice(target); // Add the target
        combined.push(b'#'); // Add a separator
        combined.extend_from_slice(data); // Add the data

        let data_length = data.len(); // Size of the data
        let target_length = target.len(); // Size of the target

        let lps = Self::longest_prefix_suffix(&combined); // Get the prefix

        // Check for occurrences
        let mut occurrences = vec![];
        for i in (target_length + 1)..=(data_length + target_length) {
            if lps[i] == target_length {
                occurrences.push(i - 2 * target_length); // Calculate start index of the occurrence
            }
        }

        occurrences // Return the array of occurrences
    }

    fn longest_prefix_suffix(data: &[u8]) -> Vec<usize> {
        let n: usize = data.len(); // Length of the data
        let mut prefix_suffix = vec![0; n]; // Initialize the prefix suffix array

        for i in 1..n {
            let mut j = prefix_suffix[i - 1]; // Get the last valid prefix length
            while j > 0 && data[i] != data[j] {
                j = prefix_suffix[j - 1]; // Backtrack
            }
            if data[i] == data[j] {
                j += 1; // If characters match, extend the prefix
            }
            prefix_suffix[i] = j; // Update the prefix
        }

        prefix_suffix // Return the prefix suffix array
    }
}
