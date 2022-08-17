#[cfg(feature = "alloc")]
#[test]
fn test_error_display_is_non_empty() {
    use alloc::string::ToString;
    use alloc::vec::Vec;

    use crate::Error;

    let try_reserve_error = Vec::<u8>::new().try_reserve(usize::MAX).unwrap_err();

    assert!(!Error::InvalidTime.to_string().is_empty());
    assert!(!Error::InvalidFormatString.to_string().is_empty());
    assert!(!Error::FormattedStringTooLarge.to_string().is_empty());
    assert!(!Error::WriteZero.to_string().is_empty());
    assert!(!Error::FmtError.to_string().is_empty());
    assert!(!Error::OutOfMemory(try_reserve_error).to_string().is_empty());
}

#[cfg(feature = "alloc")]
#[test]
fn test_error_from_try_reserve_error_is_out_of_memory_variant() {
    use alloc::vec::Vec;

    use crate::Error;

    let try_reserve_error = Vec::<u8>::new().try_reserve(usize::MAX).unwrap_err();
    assert!(matches!(try_reserve_error.into(), Error::OutOfMemory(..)));
}

#[cfg(feature = "std")]
#[test]
fn test_error_source_returns_inner_error() {
    use alloc::vec::Vec;
    use std::error::Error as _;

    use crate::Error;

    let try_reserve_error = Vec::<u8>::new().try_reserve(usize::MAX).unwrap_err();

    // Errors variants without inner error
    assert!(Error::InvalidTime.source().is_none());
    assert!(Error::InvalidFormatString.source().is_none());
    assert!(Error::FormattedStringTooLarge.source().is_none());
    assert!(Error::WriteZero.source().is_none());
    assert!(Error::FmtError.source().is_none());

    // Error variants with inner error
    let err = Error::OutOfMemory(try_reserve_error.clone());
    let err_source = err.source().unwrap().downcast_ref();
    assert_eq!(err_source, Some(&try_reserve_error));
}
