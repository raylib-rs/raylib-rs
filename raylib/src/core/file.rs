//! File manipulation functions. Should be parity with [`std::fs`] except on emscripten
use crate::{core::RaylibHandle, ffi};
use std::ffi::{CStr, CString, OsString, c_char};

/// Iterator over file paths
///
/// Created by [`FilePathList::iter`].
#[derive(Debug, Clone)]
pub struct FilePathIter<'a> {
    iter: std::slice::Iter<'a, Option<&'a c_char>>,
}
impl<'a> FilePathIter<'a> {
    /// # Safety
    /// The memory pointed to by `list` must not be mutated for `'a`.
    /// Every `*mut c_char` in `list` must outlive `'a`.
    ///
    /// ## Examples
    ///
    /// The following is invalid, because `list` is dropped while `it` is still borrowing it.
    /// ```compile_fail
    /// # use raylib::{ffi, file::*};
    /// # use std::{mem::ManuallyDrop, ffi::CStr};
    /// let mut it;
    /// let s;
    /// {
    ///     let mut paths = [
    ///         CStr::from_bytes_with_nul(b"apple\0").unwrap().as_ptr().cast_mut(),
    ///     ];
    ///     let mut list = ManuallyDrop::new(unsafe {
    ///         FilePathList::from_raw(ffi::FilePathList {
    ///             capacity: 1,
    ///             count: 1,
    ///             paths: paths.as_mut_ptr(),
    ///         })
    ///     });
    ///     it = list.iter(); // expect error[E0597]
    ///     //   ^^^^ borrowed value does not live long enough
    ///     s = it.next();
    ///     assert_eq!(s, Some("apple"));
    /// } // `list` dropped here while still borrowed
    /// assert_eq!(s, Some("apple")); // borrow later used here
    /// ```
    ///
    /// The following is invalid, because `list` is mutated while `it` is still borrowing it.
    /// ```compile_fail
    /// # use raylib::{ffi, file::*};
    /// # use std::{mem::ManuallyDrop, ffi::CStr};
    /// let mut paths = [
    ///     CStr::from_bytes_with_nul(b"apple\0").unwrap().as_ptr().cast_mut(),
    /// ];
    /// let mut list = ManuallyDrop::new(unsafe {
    ///     FilePathList::from_raw(ffi::FilePathList {
    ///         capacity: 1,
    ///         count: 1,
    ///         paths: paths.as_mut_ptr(),
    ///     })
    /// });
    /// let mut it = list.iter();
    /// //           ---- immutable borrow occurs here
    /// let s = it.next();
    /// assert_eq!(s, Some("apple"));
    /// unsafe { *(*list.paths) = b'@' as std::ffi::c_char; } // expect error[E0502]
    /// //          ^^^^ mutable borrow occurs here
    /// assert_eq!(s, Some("apple")); // immutable borrow later used here
    /// ```
    unsafe fn new(list: *mut *mut c_char, count: u32) -> Self {
        // No new items are being created that get dropped here, these are just changes in perspective of how to borrow-check the pointers.
        assert!(!list.is_null(), "file path array cannot be null");
        assert!(list.is_aligned(), "file path array must be aligned");
        let list = list.cast::<Option<&'a c_char>>();
        let iter = unsafe { std::slice::from_raw_parts(list, count as usize) }.iter();
        Self { iter }
    }
    fn func(f: Option<&'a c_char>) -> &'a str {
        // CStr isn't being "constructed", it's essentially an adapter on &[c_char]
        let s = std::slice::from_ref(f.expect("file path string cannot be null"));
        unsafe { CStr::from_ptr(s.as_ptr()) }
            .to_str()
            .expect("file path string should be in utf-8") // no???
    }
}
impl<'a> Iterator for FilePathIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().copied().map(Self::func)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    fn last(self) -> Option<Self::Item> {
        self.iter.last().copied().map(Self::func)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n).copied().map(Self::func)
    }
}
impl DoubleEndedIterator for FilePathIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().copied().map(Self::func)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n).copied().map(Self::func)
    }
}
impl ExactSizeIterator for FilePathIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

make_thin_wrapper!(FilePathList, ffi::FilePathList, ffi::UnloadDirectoryFiles);
make_thin_wrapper!(
    DroppedFilePathList,
    ffi::FilePathList,
    ffi::UnloadDroppedFiles
);

impl FilePathList {
    /// Length of the file path list
    #[inline]
    #[must_use]
    pub const fn count(&self) -> u32 {
        self.0.count
    }
    /// The amount of files that can be held in this list.
    #[inline]
    #[must_use]
    pub const fn capacity(&self) -> u32 {
        self.0.capacity
    }
    /// The paths held in this list.
    /// This function is NOT constant and the inner array will be copied into the returned Vec every time you call this.
    ///
    /// # Panics
    ///
    /// This method will panic if a path is not compatible with utf-8.
    //  which happens a lot??? WTF-8 is standard on windows!!
    #[must_use]
    pub fn paths(&self) -> Vec<&str> {
        unsafe { std::slice::from_raw_parts(self.0.paths, self.count() as usize) }
            .iter()
            .map(|f| {
                unsafe { CStr::from_ptr(*f) }
                    .to_str()
                    .expect("file path string should be in utf-8") // no???
            })
            .collect()
    }
    /// An iterator over the paths held in this list.
    #[must_use]
    pub fn iter(&self) -> FilePathIter<'_> {
        unsafe { FilePathIter::new(self.0.paths, self.count()) }
    }
}

impl<'a> IntoIterator for &'a FilePathList {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = FilePathIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl DroppedFilePathList {
    /// Length of the file path list
    #[inline]
    #[must_use]
    pub const fn count(&self) -> u32 {
        self.0.count
    }
    /// The amount of files that can be held in this list.
    #[inline]
    #[must_use]
    pub const fn capacity(&self) -> u32 {
        self.0.capacity
    }
    /// The paths held in this list.
    /// This function is NOT constant and the inner array will be copied into the returned [`Vec`] every time you call this.
    ///
    /// # Panics
    ///
    /// This method will panic if a path is not compatible with utf-8.
    #[must_use]
    pub fn paths(&self) -> Vec<&str> {
        unsafe { std::slice::from_raw_parts(self.0.paths, self.count() as usize) }
            .iter()
            .map(|f| {
                unsafe { CStr::from_ptr(*f) }
                    .to_str()
                    .expect("file path string should be in utf-8") // no???
            })
            .collect()
    }
    /// An iterator over the paths held in this list.
    #[must_use]
    pub fn iter(&self) -> FilePathIter<'_> {
        unsafe { FilePathIter::new(self.0.paths, self.count()) }
    }
}

impl<'a> IntoIterator for &'a DroppedFilePathList {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = FilePathIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl RaylibHandle {
    /// Checks if a file has been dropped into the window.
    #[inline]
    #[must_use]
    pub fn is_file_dropped(&self) -> bool {
        unsafe { ffi::IsFileDropped() }
    }

    /// Checks a file's extension.
    ///
    /// # Panics
    ///
    /// This method will panic if `file_name` or `file_ext`, when converted to a lossy string, has an internal 0 byte.
    #[inline]
    #[must_use]
    pub fn is_file_extension<A, B>(&self, file_name: A, file_ext: B) -> bool
    where
        A: Into<OsString>,
        B: Into<OsString>,
    {
        let file_name = CString::new(file_name.into().to_string_lossy().as_bytes())
            .expect("lossy file_name string should not have an internal 0 byte");
        let file_ext = CString::new(file_ext.into().to_string_lossy().as_bytes())
            .expect("lossy file_ext string should not have an internal 0 byte");
        unsafe { ffi::IsFileExtension(file_name.as_ptr(), file_ext.as_ptr()) }
    }
    /// Get the directory of the running application.
    ///
    /// # Panics
    ///
    /// This method will panic if the application directory is not compatible with utf-8.
    #[must_use]
    pub fn application_directory(&self) -> String {
        let st = unsafe { ffi::GetApplicationDirectory() };
        let c_str = unsafe { CStr::from_ptr(st) };

        // If this ever errors out, yell at @ioi_xd on Discord,
        c_str
            .to_str()
            .expect("application directory string should be in utf-8") // no???
            .to_string()
    }

    /// Get file length in bytes.
    ///
    /// # Panics
    ///
    /// This method will panic if `filename` contains an internal 0 byte.
    pub fn get_file_length<A>(&self, filename: A) -> i32
    where
        A: Into<OsString>,
    {
        let c_str = CString::new(filename.into().to_string_lossy().as_bytes())
            .expect("lossy filename string should not contain an internal 0 byte");
        unsafe { ffi::GetFileLength(c_str.as_ptr()) }
    }

    /// Check if a given path is a file or a directory
    ///
    /// # Panics
    ///
    /// This method will panic if `filename` contains an internal 0 byte.
    #[must_use]
    pub fn is_path_file<A>(&self, filename: A) -> bool
    where
        A: Into<OsString>,
    {
        let c_str = CString::new(filename.into().to_string_lossy().as_bytes())
            .expect("lossy filename string should not contain an internal 0 byte");
        unsafe { ffi::IsPathFile(c_str.as_ptr()) }
    }

    /// Load directory filepaths
    ///
    /// # Panics
    ///
    /// This method will panic if `dir_path` contains an internal 0 byte.
    pub fn load_directory_files<A>(&self, dir_path: A) -> FilePathList
    where
        A: Into<OsString>,
    {
        let c_str = CString::new(dir_path.into().to_string_lossy().as_bytes())
            .expect("lossy dir_path string should not contain an internal 0 byte");
        FilePathList(unsafe { ffi::LoadDirectoryFiles(c_str.as_ptr()) })
    }

    /// Load directory filepaths with extension filtering and recursive directory scan
    ///
    /// # Panics
    ///
    /// This method will panic if `dir_c_str` contains an internal 0 byte
    pub fn load_directory_files_ex<A>(
        &self,
        dir_path: A,
        mut filter: String,
        scan_sub_dirs: bool,
    ) -> FilePathList
    where
        A: Into<OsString>,
    {
        let dir_c_str = CString::new(dir_path.into().to_string_lossy().as_bytes())
            .expect("lossy dir_path string should not contain an internal 0 byte");
        while let Some(pos) = filter.rfind('\0') {
            filter.remove(pos);
        }
        let filter_c_str =
            CString::new(filter.as_bytes()).expect("removing all 0 bytes from the string guarantees there aren't any 0 bytes in the string");
        FilePathList(unsafe {
            ffi::LoadDirectoryFilesEx(dir_c_str.as_ptr(), filter_c_str.as_ptr(), scan_sub_dirs)
        })
    }

    /// Check if a file has been dropped into window
    #[inline]
    #[must_use]
    pub fn load_dropped_files(&self) -> DroppedFilePathList {
        unsafe { DroppedFilePathList(ffi::LoadDroppedFiles()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::ManuallyDrop;

    #[test]
    #[should_panic(expected = "file path array cannot be null")]
    fn test_null_list() {
        let list = ManuallyDrop::new(FilePathList(ffi::FilePathList {
            capacity: 0,
            count: 0,
            paths: std::ptr::null_mut(),
        }));
        let _it = list.iter();
        // should have panicked while calling .iter()
    }

    #[test]
    #[should_panic(expected = "file path string cannot be null")]
    fn test_null_item() {
        let mut paths = [std::ptr::null_mut()];
        let list = ManuallyDrop::new(FilePathList(ffi::FilePathList {
            capacity: 1,
            count: 1,
            paths: paths.as_mut_ptr(),
        }));
        let mut it = list.iter();
        let _f = it.next();
        // should have panicked while calling .next()
    }

    #[test]
    #[should_panic(expected = "file path string cannot be null")]
    fn test_null_item_double_ended() {
        let mut paths = [std::ptr::null_mut()];
        let list = ManuallyDrop::new(FilePathList(ffi::FilePathList {
            capacity: 1,
            count: 1,
            paths: paths.as_mut_ptr(),
        }));
        let mut it = list.iter();
        let _f = it.next_back();
        // should have panicked while calling .next_back()
    }

    #[test]
    fn test_len() {
        let mut paths = [
            c"apple".as_ptr().cast_mut(),
            c"orange".as_ptr().cast_mut(),
            c"banana".as_ptr().cast_mut(),
            c"mango".as_ptr().cast_mut(),
            c"pineapple".as_ptr().cast_mut(),
        ];
        let list = ManuallyDrop::new(FilePathList(ffi::FilePathList {
            capacity: 5,
            count: 5,
            paths: paths.as_mut_ptr(),
        }));
        let mut it = list.iter();
        assert_eq!(it.len(), 5);
        assert_eq!(it.next(), Some("apple"));
        assert_eq!(it.len(), 4);
        assert_eq!(it.next(), Some("orange"));
        assert_eq!(it.len(), 3);
        assert_eq!(it.next(), Some("banana"));
        assert_eq!(it.len(), 2);
        assert_eq!(it.next(), Some("mango"));
        assert_eq!(it.len(), 1);
        assert_eq!(it.next(), Some("pineapple"));
        assert_eq!(it.len(), 0);
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_len_double_ended() {
        let mut paths = [
            c"apple".as_ptr().cast_mut(),
            c"orange".as_ptr().cast_mut(),
            c"banana".as_ptr().cast_mut(),
            c"mango".as_ptr().cast_mut(),
            c"pineapple".as_ptr().cast_mut(),
        ];
        let list = ManuallyDrop::new(FilePathList(ffi::FilePathList {
            capacity: 5,
            count: 5,
            paths: paths.as_mut_ptr(),
        }));
        let mut it = list.iter();
        assert_eq!(it.len(), 5);
        assert_eq!(it.next_back(), Some("pineapple"));
        assert_eq!(it.len(), 4);
        assert_eq!(it.next_back(), Some("mango"));
        assert_eq!(it.len(), 3);
        assert_eq!(it.next_back(), Some("banana"));
        assert_eq!(it.len(), 2);
        assert_eq!(it.next_back(), Some("orange"));
        assert_eq!(it.len(), 1);
        assert_eq!(it.next_back(), Some("apple"));
        assert_eq!(it.len(), 0);
        assert_eq!(it.next_back(), None);
    }
}
