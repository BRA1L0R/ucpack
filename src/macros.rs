/// Internal usage only
macro_rules! unimpl {
    (name = $name:expr) => {{
        return Err(UcPackError::NoSupport($name));
    }};

    ($func:tt) => {
        fn $func(self) -> Result<Self::Ok, Self::Error> {
            Err(UcPackError::NoSupport(""))
        }
    };

    ($func:ident, $type:ty) => {
        fn $func(self, _: $type) -> Result<Self::Ok, Self::Error> {
            Err(UcPackError::NoSupport(core::any::type_name::<$type>()))
        }
    };
}

macro_rules! unimpl_de {
    ($func:ident, $type:ty) => {
        fn $func<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            unimpl!(name = core::any::type_name::<$type>())
        }
    };
    ($func:ident, name = $name:expr) => {
        fn $func<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            unimpl!(name = $name)
        }
    };
}

pub(crate) use unimpl;
pub(crate) use unimpl_de;
