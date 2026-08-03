use super::*;

pub(super) mod pool;
use pool::StateId;

pub(super) mod runtime;

pub(crate) enum State {
    Value(Box<dyn Any>),
    Runner(Runner),
    Mapper(Box<dyn Mapper>),
}

impl State {
    pub(crate) fn as_value(&self) -> Option<&Box<dyn Any>> {
        match self {
            Self::Value(e) => Some(e),
            _ => None,
        }
    }

    pub(crate) fn as_mut_value(&mut self) -> Option<&mut Box<dyn Any>> {
        match self {
            Self::Value(e) => Some(e),
            _ => None,
        }
    }

    pub(super) fn as_runner(&self) -> Option<&Runner> {
        match self {
            Self::Runner(e) => Some(e),
            _ => None,
        }
    }

    pub(crate) fn as_mapper(&self) -> Option<&Box<dyn Mapper>> {
        match self {
            Self::Mapper(e) => Some(e),
            _ => None,
        }
    }
}

pub(crate) enum Runner {
    /// Without input; use batch & cite by default
    Citer {
        runner: Box<dyn Fn() -> ()>,
        /// if it's an hail-citer, it works a bit different:
        /// 1) set cite-rel on insertion
        /// 2) not using cite
        is_hail_sender: bool,
    },
    /// With input; Do not use cite when run; (still use batch)
    Executer(Box<dyn Fn(Box<dyn Any>) -> Box<dyn Any>>),
}

impl Runner {
    const fn is_citer(&self) -> bool {
        match self {
            Self::Citer { .. } => true,
            _ => false,
        }
    }
}

/// Abstract Mapper Trait
pub(crate) trait Mapper {
    fn map_ref<'a>(&self, source: &'a dyn Any) -> Option<&'a dyn Any>;
    fn map_mut<'a>(&self, source: &'a mut dyn Any) -> Option<&'a mut dyn Any>;
}
