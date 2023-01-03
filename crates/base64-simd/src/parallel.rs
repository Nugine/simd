use crate::encode::encoded_length_unchecked;
use crate::{Base64, Config, Error, Extra, Out};

use rayon::prelude::{IndexedParallelIterator, ParallelIterator};
use rayon::slice::{ParallelSlice, ParallelSliceMut};
use vsimd::tools::slice_mut;

impl Base64 {
    /// **EXPERIMENTAL**:
    /// Encodes bytes to a base64 string in parallel.
    ///
    /// # Errors
    /// This function returns `Err` if the length of `dst` is not enough.
    #[cfg_attr(docsrs, doc(cfg(feature = "parallel")))]
    #[inline]
    pub fn par_encode<'d>(&self, src: &[u8], dst: Out<'d, [u8]>) -> Result<&'d mut [u8], Error> {
        let p = rayon::current_num_threads();
        let b = src.len() / 3;
        if src.len() < p * 4096 || p < 2 || b < p {
            return self.encode(src, dst);
        }

        let encoded_len = encoded_length_unchecked(src.len(), self.config);
        let dst = unsafe { dst.into_uninit_slice() };
        let dst = &mut dst[..encoded_len]; // panic?

        let chunks = (b + p) / p;

        let src_chunks = src.par_chunks(chunks * 3);
        let dst_chunks = dst.par_chunks_mut(chunks * 4);

        if self.config.extra.padding() {
            let no_pad = Config {
                kind: self.config.kind,
                extra: Extra::NoPad,
            };
            src_chunks.zip(dst_chunks).for_each(|(s, d)| unsafe {
                let len = s.len();
                let sp = s.as_ptr();
                let dp = d.as_mut_ptr().cast::<u8>();
                if len % 3 == 0 {
                    crate::multiversion::encode::auto(sp, len, dp, no_pad);
                } else {
                    crate::multiversion::encode::auto(sp, len, dp, self.config);
                }
            });
        } else {
            src_chunks.zip(dst_chunks).for_each(|(s, d)| unsafe {
                let len = s.len();
                let sp = s.as_ptr();
                let dp = d.as_mut_ptr().cast::<u8>();
                crate::multiversion::encode::auto(sp, len, dp, self.config);
            });
        }

        unsafe {
            let len = dst.len();
            let ptr = dst.as_mut_ptr().cast::<u8>();
            Ok(slice_mut(ptr, len))
        }
    }
}
