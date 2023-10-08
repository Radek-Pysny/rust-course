/// is_palindrome_vector predicate check whether the given vector is predicate or not.
fn is_palindrome_vector<T: Ord>(array: &Vec<T>) -> bool {
    let mut l = 0;
    let mut r = array.len() - 1;
    while l < r {
        if array[l] != array[r] {
            return false;
        }
        l += 1;
        r -= 1;
    }

    true
}

/// make_palindrome construct a palindrome from any given vector.
fn make_palindrome<T: Ord + Copy>(array: &Vec<T>) -> Vec<T> {
    let r_max: i32 = (array.len() as i32) - 1;
    let mut l_min = 0;
    let mut l;
    let mut r;
    let mut same = true;
    while l_min <= r_max {
        l = l_min;
        r = r_max;
        'inner: while l <= r {
            same = array[l as usize] == array[r as usize];
            if !same {
                break 'inner;
            }
            l += 1;
            r -= 1;
        }

        if same {
            let mut suffix = array.iter()
                .cloned()
                .take(l_min as usize)
                .rev()
                .collect::<Vec<_>>();
            let mut result = array.iter().cloned().collect::<Vec<_>>();
            result.append(&mut suffix);
            return result;
        }

        l_min += 1;
    }

    array.iter().cloned().collect::<Vec<_>>()    // shall never happen
}

/// palindromize construct a palindrome for the given string.
pub fn palindromize(text: &str) -> String {
    let vec = text.chars().collect::<Vec<char>>();
    if is_palindrome_vector(&vec) {
        String::from(text)
    } else {
        make_palindrome(&vec).into_iter().collect::<String>()
    }
}

#[cfg(test)]
mod tests {
    use super::{make_palindrome, palindromize};

    #[test]
    fn test_make_palindrome_empty() {
        let original: Vec<i32> = vec![];
        let result = make_palindrome(&original);

        assert_eq!(result, original);
    }

    #[test]
    fn test_make_palindrome_ok_single_char() {
        let original = vec![1];
        let result = make_palindrome(&original);

        assert_eq!(result, original);
    }

    #[test]
    fn test_make_palindrome_ok_odd_length() {
        let original = vec![3, 6, 9, 6, 3];
        let result = make_palindrome(&original);

        assert_eq!(result, original);
    }

    #[test]
    fn test_make_palindrome_ok_even_length() {
        let original = vec![2, 4, 4, 2];
        let result = make_palindrome(&original);

        assert_eq!(result, original);
    }

    #[test]
    fn test_make_palindrome_diff_odd_length() {
        let result = make_palindrome(&vec![3, 6, 9]);
        let expected = vec![3, 6, 9, 6, 3];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_make_palindrome_diff_even_length() {
        let result = make_palindrome(&vec![2, 4, 8, 16]);
        let expected = vec![2, 4, 8, 16, 8, 4, 2];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_palindromize_ok() {
        let original = String::from("jeblbej");
        let result = palindromize(&original);

        assert_eq!(result, original);
    }

    #[test]
    fn test_palindromize_diff() {
        let result = palindromize(&String::from("abcdecba"));
        let expected = String::from("abcdecbabcedcba");

        assert_eq!(result, expected);
    }
}
