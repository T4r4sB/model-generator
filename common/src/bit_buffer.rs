
/// Uses some features of BTreeMap, but faster
/// For put_range:
/// On 1000 elements x25 times faster
/// On 10_000 elements x5.4 times faster
/// On 100_000 elements the same speed
/// For remove_number_get_adj:
/// On 1000-100000 elements x50-x100 times faster
#[derive(Debug, Clone)]
pub struct BitBuffer {
  elements: Vec<usize>,
}

impl BitBuffer {
  pub fn new(len: usize) -> Self {
    let bsz = usize::BITS as usize;
    let len = len.div_ceil(bsz);
    let mut elements = Vec::new();
    elements.resize(len, 0);
    Self { elements }
  }

  pub fn clear(&mut self) {
    for e in &mut self.elements {
      *e = 0;
    }
  }

  pub fn put_number(&mut self, number: usize) {
    let mask = usize::BITS as usize - 1;
    let bsz = usize::BITS as usize;
    let n = number / bsz;
    let nn = number & mask;
    self.elements[n] |= 1 << nn;
  }

  pub fn remove_number(&mut self, number: usize) {
    let mask = usize::BITS as usize - 1;
    let bsz = usize::BITS as usize;
    let n = number / bsz;
    let nn = number & mask;
    self.elements[n] &= !(1 << nn);
  }

  pub fn remove_number_get_adj(&mut self, number: usize) -> (usize, usize) {
    let mask = usize::BITS as usize - 1;
    let bsz = usize::BITS as usize;
    let n = number / bsz;
    let nn = number & mask;
    self.elements[n] &= !(1 << nn);
    let mut result = (0, 0);
    let mut check = self.elements[n] & !((1 << nn) - 1);
    let mut i = n;
    loop {
      if check != 0 {
        result.1 = check.trailing_zeros() as usize + i * bsz;
        break;
      }
      i += 1;
      check = self.elements[i];
    }
    let mut check = self.elements[n] & ((1 << nn) - 1);
    let mut i = n;
    loop {
      if check != 0 {
        result.0 = 63 - check.leading_zeros() as usize + i * bsz;
        break;
      }
      i -= 1;
      check = self.elements[i];
    }

    result
  }

  pub fn upper_bound(&mut self, number: usize, if_fail: usize) -> usize {
    let mask = usize::BITS as usize - 1;
    let bsz = usize::BITS as usize;
    let mut n = number / bsz;
    let nn = number & mask;
    let mut check = self.elements[n] & !((usize::MAX << nn) << 1);
    loop {
      if check != 0 {
        return 63 - check.leading_zeros() as usize + n * bsz;
      }
      if n == 0 {
        return if_fail;
      }
      n -= 1;
      check = self.elements[n];
    }
  }

  pub fn put_range(
    &mut self,
    mut begin: usize,
    mut end: usize,
    use_result: bool,
    result: &mut Vec<usize>,
  ) {
    if begin > end {
      (begin, end) = (end, begin);
    }

    let mask = usize::BITS as usize - 1;
    let bsz = usize::BITS as usize;
    let b = begin / bsz;
    let bn = begin & mask;
    let e = end / bsz;
    let en = end & mask;

    let mut push_bits = |pat: &mut usize, mask: usize, new_bits: usize, offset: usize| {
      if use_result {
        let mut p = *pat & mask;
        while p != 0 {
          let bit = p.trailing_zeros() as usize;
          result.push(offset + bit);
          p -= 1 << bit;
        }
      }
      *pat = *pat & !mask | new_bits;
    };

    if b == e {
      let be_mask = (1 << en) - (1 << (bn + 1));
      push_bits(&mut self.elements[b], be_mask, 1 << bn | 1 << en, b * bsz);
      self.elements[b] &= !be_mask;
      self.elements[b] |= 1 << bn | 1 << en;
    } else {
      let b_mask = (usize::MAX << bn) << 1;
      push_bits(&mut self.elements[b], b_mask, 1 << bn, b * bsz);
      for cb in b + 1..e {
        push_bits(&mut self.elements[cb], usize::MAX, 0, cb * bsz);
      }
      let e_mask = (1 << en) - 1;
      push_bits(&mut self.elements[e], e_mask, 1 << en, e * bsz);
    }
  }
}
