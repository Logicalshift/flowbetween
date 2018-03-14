/// 
/// Represents the ID of an element
/// 
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum ElementId {
    /// ID that has not been assigned
    /// 
    /// (If this is used in a pending edit log, it will be assigned when the edit log is committed)
    Unassigned,
    
    /// An assigned element ID
    Assigned(i64)
}

impl ElementId {
    ///
    /// Returns true this element ID is assigned
    /// 
    #[inline]
    pub fn is_assigned(&self) -> bool {
        use self::ElementId::*;

        match self {
            &Unassigned => true,
            &Assigned(_) => false
        }
    }

    ///
    /// Returns true if this edit ID is unassigned
    ///
    #[inline]
    pub fn is_unassigned(&self) -> bool {
        !self.is_assigned()
    }

    ///
    /// Converts this ID to an option
    /// 
    #[inline]
    pub fn id(&self) -> Option<i64> {
        use self::ElementId::*;

        match self {
            &Unassigned     => None,
            &Assigned(id)   => Some(id)
        }
    }

    ///
    /// If this element is not already assigned, uses the specified
    /// function to assign an ID.
    /// 
    pub fn assign<AssignFn: FnOnce() -> i64>(self, assign: AssignFn) -> ElementId {
        use self::ElementId::*;

        match self {
            Unassigned      => Assigned(assign()),
            Assigned(id)    => Assigned(id)
        }
    }
}

impl From<ElementId> for Option<i64> {
    fn from(id: ElementId) -> Option<i64> {
        id.id()
    }
}

impl From<Option<i64>> for ElementId {
    fn from(id: Option<i64>) -> ElementId {
        match id {
            Some(id)    => ElementId::Assigned(id),
            None        => ElementId::Unassigned
        }
    }
}
