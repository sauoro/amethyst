#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum Reliability {
    #[default]
    Unreliable,
    UnreliableSequenced,
    Reliable,
    ReliableOrdered,
    ReliableSequenced,
    UnreliableWithAckReceipt,
    ReliableWithAckReceipt,
    ReliableOrderedWithAckReceipt,
}

impl Reliability {
    
    #[inline]
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Reliability::Unreliable),
            1 => Some(Reliability::UnreliableSequenced),
            2 => Some(Reliability::Reliable),
            3 => Some(Reliability::ReliableOrdered),
            4 => Some(Reliability::ReliableSequenced),
            5 => Some(Reliability::UnreliableWithAckReceipt),
            6 => Some(Reliability::ReliableWithAckReceipt),
            7 => Some(Reliability::ReliableOrderedWithAckReceipt),
            _ => None,
        }
    }

    #[inline]
    pub const fn is_reliable(&self) -> bool {
        matches!(
            self,
            Reliability::Reliable
                | Reliability::ReliableOrdered
                | Reliability::ReliableSequenced
                | Reliability::ReliableWithAckReceipt
                | Reliability::ReliableOrderedWithAckReceipt
        )
    }

    #[inline]
    pub const fn is_ordered(&self) -> bool {
        matches!(
            self,
            Reliability::ReliableOrdered | Reliability::ReliableOrderedWithAckReceipt
        )
    }

    #[inline]
    pub const fn is_sequenced(&self) -> bool {
        matches!(
            self,
            Reliability::UnreliableSequenced | Reliability::ReliableSequenced
        )
    }

    #[inline]
    pub const fn is_ordered_or_sequenced(&self) -> bool {
        self.is_ordered() || self.is_sequenced()
    }

    #[inline]
    pub const fn is_ack_receipt(&self) -> bool {
        matches!(
            self,
            Reliability::UnreliableWithAckReceipt
                | Reliability::ReliableWithAckReceipt
                | Reliability::ReliableOrderedWithAckReceipt
        )
    }
}