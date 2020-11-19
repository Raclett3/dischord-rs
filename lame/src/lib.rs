use std::ops::Drop;
use std::os::raw::{c_int, c_void};

pub type LamePtr = *mut c_void;

#[link(name = "mp3lame")]
extern "C" {
    pub fn lame_init() -> LamePtr;
    pub fn lame_close(ptr: LamePtr) -> c_int;
    pub fn lame_set_in_samplerate(ptr: LamePtr, in_samplerate: c_int) -> c_int;
    pub fn lame_get_in_samplerate(ptr: LamePtr) -> c_int;
    pub fn lame_set_out_samplerate(ptr: LamePtr, out_samplerate: c_int) -> c_int;
    pub fn lame_get_out_samplerate(ptr: LamePtr) -> c_int;
    pub fn lame_set_num_channels(ptr: LamePtr, channels: c_int) -> c_int;
    pub fn lame_get_num_channels(ptr: LamePtr) -> c_int;
    pub fn lame_set_quality(ptr: LamePtr, quality: c_int) -> c_int;
    pub fn lame_get_quality(ptr: LamePtr) -> c_int;
    pub fn lame_set_brate(ptr: LamePtr, brate: c_int) -> c_int;
    pub fn lame_get_brate(ptr: LamePtr) -> c_int;
    pub fn lame_init_params(ptr: LamePtr) -> c_int;
    pub fn lame_encode_buffer(
        ptr: LamePtr,
        leftpcm: *const i16,
        rightpcm: *const i16,
        num_samples: c_int,
        mp3buffer: *mut u8,
        mp3buffer_size: c_int,
    ) -> c_int;
}

#[derive(Debug)]
pub enum Error {
    GenericError,
    NoMem,
    BadBitRate,
    BadSampleFreq,
    InternalError,
    Unknown(c_int),
}

fn int_to_result(ret: c_int) -> Result<(), Error> {
    match ret {
        0 => Ok(()),
        -1 => Err(Error::GenericError),
        -10 => Err(Error::NoMem),
        -11 => Err(Error::BadBitRate),
        -12 => Err(Error::BadSampleFreq),
        -13 => Err(Error::InternalError),
        err => Err(Error::Unknown(err)),
    }
}

pub struct Lame {
    ptr: LamePtr,
}

impl Lame {
    pub fn init() -> Result<Lame, Error> {
        let ctx = unsafe { lame_init() };

        if ctx.is_null() {
            panic!("lame_init() returned null");
        }

        int_to_result(unsafe { lame_init_params(ctx) })?;
        Ok(Lame { ptr: ctx })
    }

    pub fn samplerate_in(&self) -> i32 {
        unsafe { lame_get_in_samplerate(self.ptr) as i32 }
    }

    pub fn set_samplerate_in(&mut self, sample_rate: i32) -> Result<(), Error> {
        int_to_result(unsafe { lame_set_in_samplerate(self.ptr, sample_rate as c_int) })
    }

    pub fn samplerate_out(&self) -> i32 {
        unsafe { lame_get_out_samplerate(self.ptr) as i32 }
    }

    pub fn set_samplerate_out(&mut self, sample_rate: i32) -> Result<(), Error> {
        int_to_result(unsafe { lame_set_out_samplerate(self.ptr, sample_rate as c_int) })
    }

    pub fn set_samplerate(&mut self, sample_rate: i32) -> Result<(), Error> {
        self.set_samplerate_in(sample_rate)
            .and_then(|_| self.set_samplerate_out(sample_rate))
    }

    pub fn channels(&self) -> i32 {
        unsafe { lame_get_num_channels(self.ptr) as i32 }
    }

    pub fn set_channels(&mut self, channels: u8) -> Result<(), Error> {
        int_to_result(unsafe { lame_set_num_channels(self.ptr, channels as c_int) })
    }

    pub fn quality(&self) -> i32 {
        unsafe { lame_get_quality(self.ptr) as i32 }
    }

    pub fn set_quality(&mut self, quality: i32) -> Result<(), Error> {
        int_to_result(unsafe { lame_set_quality(self.ptr, quality as c_int) })
    }

    pub fn kilobitrate(&self) -> i32 {
        unsafe { lame_get_brate(self.ptr) as i32 }
    }

    pub fn set_kilobitrate(&mut self, quality: i32) -> Result<(), Error> {
        int_to_result(unsafe { lame_set_brate(self.ptr, quality as c_int) })
    }

    pub fn encode(&mut self, pcm_left: &[i16], pcm_right: &[i16]) -> Result<Vec<u8>, i32> {
        if pcm_left.len() != pcm_right.len() {
            panic!(
                "The number of left and right channel must be same\n  left: {}\n right: {}",
                pcm_left.len(),
                pcm_right.len()
            );
        }

        let mut mp3_buffer = Vec::new();
        mp3_buffer.resize(pcm_left.len(), 0);

        let ret = unsafe {
            lame_encode_buffer(
                self.ptr,
                pcm_left.as_ptr(),
                pcm_right.as_ptr(),
                pcm_left.len() as c_int,
                mp3_buffer.as_mut_ptr(),
                mp3_buffer.len() as c_int,
            )
        };

        match ret {
            size if size >= 0 => {
                mp3_buffer.resize(size as usize, 0);
                Ok(mp3_buffer)
            }
            err => Err(err as i32),
        }
    }

    pub fn encode_mono(&mut self, pcm: &[i16]) -> Result<Vec<u8>, i32> {
        self.encode(pcm, pcm)
    }
}

impl Drop for Lame {
    fn drop(&mut self) {
        unsafe { lame_close(self.ptr) };
    }
}
