#![feature(prelude_import)]
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std;
use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use std::env;
fn main() -> Result<()> {
    tokio::runtime::Builder::new()
        .basic_scheduler()
        .threaded_scheduler()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            {
                env_logger::init();
                let database_url =
                    env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
                let db_pool = PgPoolOptions::new()
                    .max_connections(5)
                    .connect(&database_url)
                    .await?;
                InsertClub {
                    name: "Hello".into(),
                    test1: "World".into(),
                    test2: TestEnum::Test1,
                    test3: Some(true),
                    test4: None,
                }
                .insert();
                Ok(())
            }
        })
}
#[repr(i32)]
pub enum TestEnum {
    Test1 = 1,
    Test2 = 2,
}
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(rust_2018_idioms, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for TestEnum {
        fn serialize<__S>(&self, __serializer: __S) -> _serde::export::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            match *self {
                TestEnum::Test1 => _serde::Serializer::serialize_unit_variant(
                    __serializer,
                    "TestEnum",
                    0u32,
                    "Test1",
                ),
                TestEnum::Test2 => _serde::Serializer::serialize_unit_variant(
                    __serializer,
                    "TestEnum",
                    1u32,
                    "Test2",
                ),
            }
        }
    }
};
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(rust_2018_idioms, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for TestEnum {
        fn deserialize<__D>(__deserializer: __D) -> _serde::export::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            enum __Field {
                __field0,
                __field1,
            }
            struct __FieldVisitor;
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::export::Formatter,
                ) -> _serde::export::fmt::Result {
                    _serde::export::Formatter::write_str(__formatter, "variant identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::export::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::export::Ok(__Field::__field0),
                        1u64 => _serde::export::Ok(__Field::__field1),
                        _ => _serde::export::Err(_serde::de::Error::invalid_value(
                            _serde::de::Unexpected::Unsigned(__value),
                            &"variant index 0 <= i < 2",
                        )),
                    }
                }
                fn visit_str<__E>(self, __value: &str) -> _serde::export::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "Test1" => _serde::export::Ok(__Field::__field0),
                        "Test2" => _serde::export::Ok(__Field::__field1),
                        _ => _serde::export::Err(_serde::de::Error::unknown_variant(
                            __value, VARIANTS,
                        )),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::export::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"Test1" => _serde::export::Ok(__Field::__field0),
                        b"Test2" => _serde::export::Ok(__Field::__field1),
                        _ => {
                            let __value = &_serde::export::from_utf8_lossy(__value);
                            _serde::export::Err(_serde::de::Error::unknown_variant(
                                __value, VARIANTS,
                            ))
                        }
                    }
                }
            }
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(__deserializer: __D) -> _serde::export::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            struct __Visitor<'de> {
                marker: _serde::export::PhantomData<TestEnum>,
                lifetime: _serde::export::PhantomData<&'de ()>,
            }
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = TestEnum;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::export::Formatter,
                ) -> _serde::export::fmt::Result {
                    _serde::export::Formatter::write_str(__formatter, "enum TestEnum")
                }
                fn visit_enum<__A>(
                    self,
                    __data: __A,
                ) -> _serde::export::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::EnumAccess<'de>,
                {
                    match match _serde::de::EnumAccess::variant(__data) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    } {
                        (__Field::__field0, __variant) => {
                            match _serde::de::VariantAccess::unit_variant(__variant) {
                                _serde::export::Ok(__val) => __val,
                                _serde::export::Err(__err) => {
                                    return _serde::export::Err(__err);
                                }
                            };
                            _serde::export::Ok(TestEnum::Test1)
                        }
                        (__Field::__field1, __variant) => {
                            match _serde::de::VariantAccess::unit_variant(__variant) {
                                _serde::export::Ok(__val) => __val,
                                _serde::export::Err(__err) => {
                                    return _serde::export::Err(__err);
                                }
                            };
                            _serde::export::Ok(TestEnum::Test2)
                        }
                    }
                }
            }
            const VARIANTS: &'static [&'static str] = &["Test1", "Test2"];
            _serde::Deserializer::deserialize_enum(
                __deserializer,
                "TestEnum",
                VARIANTS,
                __Visitor {
                    marker: _serde::export::PhantomData::<TestEnum>,
                    lifetime: _serde::export::PhantomData,
                },
            )
        }
    }
};
impl ::core::marker::StructuralPartialEq for TestEnum {}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::cmp::PartialEq for TestEnum {
    #[inline]
    fn eq(&self, other: &TestEnum) -> bool {
        {
            let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
            let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
            if true && __self_vi == __arg_1_vi {
                match (&*self, &*other) {
                    _ => true,
                }
            } else {
                false
            }
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::cmp::PartialOrd for TestEnum {
    #[inline]
    fn partial_cmp(&self, other: &TestEnum) -> ::core::option::Option<::core::cmp::Ordering> {
        {
            let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
            let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
            if true && __self_vi == __arg_1_vi {
                match (&*self, &*other) {
                    _ => ::core::option::Option::Some(::core::cmp::Ordering::Equal),
                }
            } else {
                __self_vi.partial_cmp(&__arg_1_vi)
            }
        }
    }
}
impl ::core::marker::StructuralEq for TestEnum {}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::cmp::Eq for TestEnum {
    #[inline]
    #[doc(hidden)]
    fn assert_receiver_is_total_eq(&self) -> () {
        {}
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::cmp::Ord for TestEnum {
    #[inline]
    fn cmp(&self, other: &TestEnum) -> ::core::cmp::Ordering {
        {
            let __self_vi = unsafe { ::core::intrinsics::discriminant_value(&*self) };
            let __arg_1_vi = unsafe { ::core::intrinsics::discriminant_value(&*other) };
            if true && __self_vi == __arg_1_vi {
                match (&*self, &*other) {
                    _ => ::core::cmp::Ordering::Equal,
                }
            } else {
                __self_vi.cmp(&__arg_1_vi)
            }
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::clone::Clone for TestEnum {
    #[inline]
    fn clone(&self) -> TestEnum {
        {
            *self
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::marker::Copy for TestEnum {}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for TestEnum {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match (&*self,) {
            (&TestEnum::Test1,) => {
                let mut debug_trait_builder = f.debug_tuple("Test1");
                debug_trait_builder.finish()
            }
            (&TestEnum::Test2,) => {
                let mut debug_trait_builder = f.debug_tuple("Test2");
                debug_trait_builder.finish()
            }
        }
    }
}
impl<'q, DB: sqlx::Database> sqlx::encode::Encode<'q, DB> for TestEnum
where
    i32: sqlx::encode::Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        let value = match self {
            TestEnum::Test1 => (TestEnum::Test1 as i32),
            TestEnum::Test2 => (TestEnum::Test2 as i32),
        };
        <i32 as sqlx::encode::Encode<DB>>::encode_by_ref(&value, buf)
    }
    fn size_hint(&self) -> usize {
        <i32 as sqlx::encode::Encode<DB>>::size_hint(&Default::default())
    }
}
impl<'r, DB: sqlx::Database> sqlx::decode::Decode<'r, DB> for TestEnum
where
    i32: sqlx::decode::Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> std::result::Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <i32 as sqlx::decode::Decode<'r, DB>>::decode(value)?;
        match value {
            _ if (TestEnum::Test1 as i32) == value => Ok(TestEnum::Test1),
            _ if (TestEnum::Test2 as i32) == value => Ok(TestEnum::Test2),
            _ => Err(Box::new(sqlx::Error::Decode(
                {
                    let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                        &["invalid value ", " for enum "],
                        &match (&value, &"TestEnum") {
                            (arg0, arg1) => [
                                ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt),
                                ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                            ],
                        },
                    ));
                    res
                }
                .into(),
            ))),
        }
    }
}
impl<DB: sqlx::Database> sqlx::Type<DB> for TestEnum
where
    i32: sqlx::Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <i32 as sqlx::Type<DB>>::type_info()
    }
}
#[allow(unused)]
fn enum_convert(val: &TestEnum) -> i32 {
    *val as i32
}
# [ ormx ( table = "clubs" , id = id , insertable , patchable , deletable ) ]
struct Club {
    id: i32,
    name: String,
    test1: String,
    # [ ormx ( get_optional = by_name , set = update_name ) ]
    #[ormx(custom_type, convert = "enum_convert")]
    test2: TestEnum,
    test3: Option<bool>,
    #[ormx(convert = "my_convert")]
    test4: Option<Vec<i32>>,
}
impl Club {
    async fn by_name(
        con: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
        by: &TestEnum,
    ) -> sqlx::Result<Option<Self>> {
        { { use sqlx :: Arguments as _ ; let arg0 = & ( enum_convert ( & by ) ) ; if false { use sqlx :: ty_match :: { WrapSameExt as _ , MatchBorrowExt as _ } ; let _expr = sqlx :: ty_match :: dupe_value ( arg0 ) ; let ty_check = sqlx :: ty_match :: WrapSame :: < i32 , _ > :: new ( & _expr ) . wrap_same ( ) ; let ( mut _ty_check , match_borrow ) = sqlx :: ty_match :: MatchBorrow :: new ( ty_check , & _expr ) ; _ty_check = match_borrow . match_borrow ( ) ; { { :: std :: rt :: begin_panic ( "explicit panic" ) } } ; } let mut query_args = < sqlx :: postgres :: Postgres as sqlx :: database :: HasArguments > :: default ( ) ; query_args . reserve ( 1usize , 0 + sqlx :: encode :: Encode :: < sqlx :: postgres :: Postgres > :: size_hint ( arg0 ) ) ; query_args . add ( arg0 ) ; sqlx :: query_with :: < sqlx :: postgres :: Postgres , _ > ( "SELECT id,name,test1,test2 AS \"test2: _\",test3,test4 FROM clubs WHERE test2 = $1" , query_args ) . try_map ( | row : sqlx :: postgres :: PgRow | { use sqlx :: Row as _ ; let id = row . try_get_unchecked :: < i32 , _ > ( 0usize ) ? ; let name = row . try_get_unchecked :: < String , _ > ( 1usize ) ? ; let test1 = row . try_get_unchecked :: < String , _ > ( 2usize ) ? ; let test2 = row . try_get ( 3usize ) ? ; let test3 = row . try_get_unchecked :: < Option < bool > , _ > ( 4usize ) ? ; let test4 = row . try_get_unchecked :: < Option < Vec < i32 > > , _ > ( 5usize ) ? ; Ok ( Self { id : id , name : name , test1 : test1 , test2 : test2 , test3 : test3 , test4 : test4 , } ) } ) } } . fetch_optional ( con ) . await
    }
    async fn update_name(
        &mut self,
        con: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
        value: TestEnum,
    ) -> sqlx::Result<()> {
        {
            {
                use sqlx::Arguments as _;
                let arg0 = &(enum_convert(&value));
                let arg1 = &(&self.id);
                if false {
                    use sqlx::ty_match::{WrapSameExt as _, MatchBorrowExt as _};
                    let _expr = sqlx::ty_match::dupe_value(arg0);
                    let ty_check = sqlx::ty_match::WrapSame::<i32, _>::new(&_expr).wrap_same();
                    let (mut _ty_check, match_borrow) =
                        sqlx::ty_match::MatchBorrow::new(ty_check, &_expr);
                    _ty_check = match_borrow.match_borrow();
                    {
                        {
                            ::std::rt::begin_panic("explicit panic")
                        }
                    };
                }
                if false {
                    use sqlx::ty_match::{WrapSameExt as _, MatchBorrowExt as _};
                    let _expr = sqlx::ty_match::dupe_value(arg1);
                    let ty_check = sqlx::ty_match::WrapSame::<i32, _>::new(&_expr).wrap_same();
                    let (mut _ty_check, match_borrow) =
                        sqlx::ty_match::MatchBorrow::new(ty_check, &_expr);
                    _ty_check = match_borrow.match_borrow();
                    {
                        {
                            ::std::rt::begin_panic("explicit panic")
                        }
                    };
                }
                let mut query_args =
                    <sqlx::postgres::Postgres as sqlx::database::HasArguments>::default();
                query_args.reserve(
                    2usize,
                    0 + sqlx::encode::Encode::<sqlx::postgres::Postgres>::size_hint(arg0)
                        + sqlx::encode::Encode::<sqlx::postgres::Postgres>::size_hint(arg1),
                );
                query_args.add(arg0);
                query_args.add(arg1);
                sqlx::query_with::<sqlx::postgres::Postgres, _>(
                    "UPDATE clubs SET test2 = $1 WHERE id = $2",
                    query_args,
                )
            }
        }
        .execute(con)
        .await?;
        self.test2 = value;
        Ok(())
    }
    async fn update(
        &self,
        con: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> sqlx::Result<()> {
        { { use sqlx :: Arguments as _ ; let arg0 = & ( self . id ) ; let arg1 = & ( self . name ) ; let arg2 = & ( self . test1 ) ; let arg3 = & ( enum_convert ( & self . test2 ) ) ; let arg4 = & ( self . test3 ) ; let arg5 = & ( my_convert ( & self . test4 ) ) ; if false { use sqlx :: ty_match :: { WrapSameExt as _ , MatchBorrowExt as _ } ; let _expr = sqlx :: ty_match :: dupe_value ( arg0 ) ; let ty_check = sqlx :: ty_match :: WrapSame :: < i32 , _ > :: new ( & _expr ) . wrap_same ( ) ; let ( mut _ty_check , match_borrow ) = sqlx :: ty_match :: MatchBorrow :: new ( ty_check , & _expr ) ; _ty_check = match_borrow . match_borrow ( ) ; { { :: std :: rt :: begin_panic ( "explicit panic" ) } } ; } if false { use sqlx :: ty_match :: { WrapSameExt as _ , MatchBorrowExt as _ } ; let _expr = sqlx :: ty_match :: dupe_value ( arg1 ) ; let ty_check = sqlx :: ty_match :: WrapSame :: < & str , _ > :: new ( & _expr ) . wrap_same ( ) ; let ( mut _ty_check , match_borrow ) = sqlx :: ty_match :: MatchBorrow :: new ( ty_check , & _expr ) ; _ty_check = match_borrow . match_borrow ( ) ; { { :: std :: rt :: begin_panic ( "explicit panic" ) } } ; } if false { use sqlx :: ty_match :: { WrapSameExt as _ , MatchBorrowExt as _ } ; let _expr = sqlx :: ty_match :: dupe_value ( arg2 ) ; let ty_check = sqlx :: ty_match :: WrapSame :: < & str , _ > :: new ( & _expr ) . wrap_same ( ) ; let ( mut _ty_check , match_borrow ) = sqlx :: ty_match :: MatchBorrow :: new ( ty_check , & _expr ) ; _ty_check = match_borrow . match_borrow ( ) ; { { :: std :: rt :: begin_panic ( "explicit panic" ) } } ; } if false { use sqlx :: ty_match :: { WrapSameExt as _ , MatchBorrowExt as _ } ; let _expr = sqlx :: ty_match :: dupe_value ( arg3 ) ; let ty_check = sqlx :: ty_match :: WrapSame :: < i32 , _ > :: new ( & _expr ) . wrap_same ( ) ; let ( mut _ty_check , match_borrow ) = sqlx :: ty_match :: MatchBorrow :: new ( ty_check , & _expr ) ; _ty_check = match_borrow . match_borrow ( ) ; { { :: std :: rt :: begin_panic ( "explicit panic" ) } } ; } if false { use sqlx :: ty_match :: { WrapSameExt as _ , MatchBorrowExt as _ } ; let _expr = sqlx :: ty_match :: dupe_value ( arg4 ) ; let ty_check = sqlx :: ty_match :: WrapSame :: < bool , _ > :: new ( & _expr ) . wrap_same ( ) ; let ( mut _ty_check , match_borrow ) = sqlx :: ty_match :: MatchBorrow :: new ( ty_check , & _expr ) ; _ty_check = match_borrow . match_borrow ( ) ; { { :: std :: rt :: begin_panic ( "explicit panic" ) } } ; } if false { use sqlx :: ty_match :: { WrapSameExt as _ , MatchBorrowExt as _ } ; let _expr = sqlx :: ty_match :: dupe_value ( arg5 ) ; let ty_check = sqlx :: ty_match :: WrapSame :: < & [ i32 ] , _ > :: new ( & _expr ) . wrap_same ( ) ; let ( mut _ty_check , match_borrow ) = sqlx :: ty_match :: MatchBorrow :: new ( ty_check , & _expr ) ; _ty_check = match_borrow . match_borrow ( ) ; { { :: std :: rt :: begin_panic ( "explicit panic" ) } } ; } let mut query_args = < sqlx :: postgres :: Postgres as sqlx :: database :: HasArguments > :: default ( ) ; query_args . reserve ( 6usize , 0 + sqlx :: encode :: Encode :: < sqlx :: postgres :: Postgres > :: size_hint ( arg0 ) + sqlx :: encode :: Encode :: < sqlx :: postgres :: Postgres > :: size_hint ( arg1 ) + sqlx :: encode :: Encode :: < sqlx :: postgres :: Postgres > :: size_hint ( arg2 ) + sqlx :: encode :: Encode :: < sqlx :: postgres :: Postgres > :: size_hint ( arg3 ) + sqlx :: encode :: Encode :: < sqlx :: postgres :: Postgres > :: size_hint ( arg4 ) + sqlx :: encode :: Encode :: < sqlx :: postgres :: Postgres > :: size_hint ( arg5 ) ) ; query_args . add ( arg0 ) ; query_args . add ( arg1 ) ; query_args . add ( arg2 ) ; query_args . add ( arg3 ) ; query_args . add ( arg4 ) ; query_args . add ( arg5 ) ; sqlx :: query_with :: < sqlx :: postgres :: Postgres , _ > ( "UPDATE clubs SET name = $2, test1 = $3, test2 = $4, test3 = $5, test4 = $6 WHERE id = $1" , query_args ) } } . execute ( con ) . await ? ;
        Ok(())
    }
    async fn delete(
        self,
        con: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    ) -> sqlx::Result<()> {
        {
            {
                use sqlx::Arguments as _;
                let arg0 = &(self.id);
                if false {
                    use sqlx::ty_match::{WrapSameExt as _, MatchBorrowExt as _};
                    let _expr = sqlx::ty_match::dupe_value(arg0);
                    let ty_check = sqlx::ty_match::WrapSame::<i32, _>::new(&_expr).wrap_same();
                    let (mut _ty_check, match_borrow) =
                        sqlx::ty_match::MatchBorrow::new(ty_check, &_expr);
                    _ty_check = match_borrow.match_borrow();
                    {
                        {
                            ::std::rt::begin_panic("explicit panic")
                        }
                    };
                }
                let mut query_args =
                    <sqlx::postgres::Postgres as sqlx::database::HasArguments>::default();
                query_args.reserve(
                    1usize,
                    0 + sqlx::encode::Encode::<sqlx::postgres::Postgres>::size_hint(arg0),
                );
                query_args.add(arg0);
                sqlx::query_with::<sqlx::postgres::Postgres, _>(
                    "DELETE FROM clubs WHERE id = $1",
                    query_args,
                )
            }
        }
        .execute(con)
        .await?;
        Ok(())
    }
}
struct InsertClub {
    name: String,
    test1: String,
    test2: TestEnum,
    test3: Option<bool>,
    test4: Option<Vec<i32>>,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for InsertClub {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            InsertClub {
                name: ref __self_0_0,
                test1: ref __self_0_1,
                test2: ref __self_0_2,
                test3: ref __self_0_3,
                test4: ref __self_0_4,
            } => {
                let mut debug_trait_builder = f.debug_struct("InsertClub");
                let _ = debug_trait_builder.field("name", &&(*__self_0_0));
                let _ = debug_trait_builder.field("test1", &&(*__self_0_1));
                let _ = debug_trait_builder.field("test2", &&(*__self_0_2));
                let _ = debug_trait_builder.field("test3", &&(*__self_0_3));
                let _ = debug_trait_builder.field("test4", &&(*__self_0_4));
                debug_trait_builder.finish()
            }
        }
    }
}
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(rust_2018_idioms, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for InsertClub {
        fn serialize<__S>(&self, __serializer: __S) -> _serde::export::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = match _serde::Serializer::serialize_struct(
                __serializer,
                "InsertClub",
                false as usize + 1 + 1 + 1 + 1 + 1,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "name",
                &self.name,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "test1",
                &self.test1,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "test2",
                &self.test2,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "test3",
                &self.test3,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "test4",
                &self.test4,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(rust_2018_idioms, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for InsertClub {
        fn deserialize<__D>(__deserializer: __D) -> _serde::export::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            enum __Field {
                __field0,
                __field1,
                __field2,
                __field3,
                __field4,
                __ignore,
            }
            struct __FieldVisitor;
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::export::Formatter,
                ) -> _serde::export::fmt::Result {
                    _serde::export::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::export::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::export::Ok(__Field::__field0),
                        1u64 => _serde::export::Ok(__Field::__field1),
                        2u64 => _serde::export::Ok(__Field::__field2),
                        3u64 => _serde::export::Ok(__Field::__field3),
                        4u64 => _serde::export::Ok(__Field::__field4),
                        _ => _serde::export::Err(_serde::de::Error::invalid_value(
                            _serde::de::Unexpected::Unsigned(__value),
                            &"field index 0 <= i < 5",
                        )),
                    }
                }
                fn visit_str<__E>(self, __value: &str) -> _serde::export::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "name" => _serde::export::Ok(__Field::__field0),
                        "test1" => _serde::export::Ok(__Field::__field1),
                        "test2" => _serde::export::Ok(__Field::__field2),
                        "test3" => _serde::export::Ok(__Field::__field3),
                        "test4" => _serde::export::Ok(__Field::__field4),
                        _ => _serde::export::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::export::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"name" => _serde::export::Ok(__Field::__field0),
                        b"test1" => _serde::export::Ok(__Field::__field1),
                        b"test2" => _serde::export::Ok(__Field::__field2),
                        b"test3" => _serde::export::Ok(__Field::__field3),
                        b"test4" => _serde::export::Ok(__Field::__field4),
                        _ => _serde::export::Ok(__Field::__ignore),
                    }
                }
            }
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(__deserializer: __D) -> _serde::export::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            struct __Visitor<'de> {
                marker: _serde::export::PhantomData<InsertClub>,
                lifetime: _serde::export::PhantomData<&'de ()>,
            }
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = InsertClub;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::export::Formatter,
                ) -> _serde::export::fmt::Result {
                    _serde::export::Formatter::write_str(__formatter, "struct InsertClub")
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::export::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 =
                        match match _serde::de::SeqAccess::next_element::<String>(&mut __seq) {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        } {
                            _serde::export::Some(__value) => __value,
                            _serde::export::None => {
                                return _serde::export::Err(_serde::de::Error::invalid_length(
                                    0usize,
                                    &"struct InsertClub with 5 elements",
                                ));
                            }
                        };
                    let __field1 =
                        match match _serde::de::SeqAccess::next_element::<String>(&mut __seq) {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        } {
                            _serde::export::Some(__value) => __value,
                            _serde::export::None => {
                                return _serde::export::Err(_serde::de::Error::invalid_length(
                                    1usize,
                                    &"struct InsertClub with 5 elements",
                                ));
                            }
                        };
                    let __field2 =
                        match match _serde::de::SeqAccess::next_element::<TestEnum>(&mut __seq) {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        } {
                            _serde::export::Some(__value) => __value,
                            _serde::export::None => {
                                return _serde::export::Err(_serde::de::Error::invalid_length(
                                    2usize,
                                    &"struct InsertClub with 5 elements",
                                ));
                            }
                        };
                    let __field3 =
                        match match _serde::de::SeqAccess::next_element::<Option<bool>>(&mut __seq)
                        {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        } {
                            _serde::export::Some(__value) => __value,
                            _serde::export::None => {
                                return _serde::export::Err(_serde::de::Error::invalid_length(
                                    3usize,
                                    &"struct InsertClub with 5 elements",
                                ));
                            }
                        };
                    let __field4 = match match _serde::de::SeqAccess::next_element::<Option<Vec<i32>>>(
                        &mut __seq,
                    ) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    } {
                        _serde::export::Some(__value) => __value,
                        _serde::export::None => {
                            return _serde::export::Err(_serde::de::Error::invalid_length(
                                4usize,
                                &"struct InsertClub with 5 elements",
                            ));
                        }
                    };
                    _serde::export::Ok(InsertClub {
                        name: __field0,
                        test1: __field1,
                        test2: __field2,
                        test3: __field3,
                        test4: __field4,
                    })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::export::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::export::Option<String> = _serde::export::None;
                    let mut __field1: _serde::export::Option<String> = _serde::export::None;
                    let mut __field2: _serde::export::Option<TestEnum> = _serde::export::None;
                    let mut __field3: _serde::export::Option<Option<bool>> = _serde::export::None;
                    let mut __field4: _serde::export::Option<Option<Vec<i32>>> =
                        _serde::export::None;
                    while let _serde::export::Some(__key) =
                        match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        }
                    {
                        match __key {
                            __Field::__field0 => {
                                if _serde::export::Option::is_some(&__field0) {
                                    return _serde::export::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("name"),
                                    );
                                }
                                __field0 = _serde::export::Some(
                                    match _serde::de::MapAccess::next_value::<String>(&mut __map) {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    },
                                );
                            }
                            __Field::__field1 => {
                                if _serde::export::Option::is_some(&__field1) {
                                    return _serde::export::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("test1"),
                                    );
                                }
                                __field1 = _serde::export::Some(
                                    match _serde::de::MapAccess::next_value::<String>(&mut __map) {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    },
                                );
                            }
                            __Field::__field2 => {
                                if _serde::export::Option::is_some(&__field2) {
                                    return _serde::export::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("test2"),
                                    );
                                }
                                __field2 = _serde::export::Some(
                                    match _serde::de::MapAccess::next_value::<TestEnum>(&mut __map)
                                    {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    },
                                );
                            }
                            __Field::__field3 => {
                                if _serde::export::Option::is_some(&__field3) {
                                    return _serde::export::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("test3"),
                                    );
                                }
                                __field3 = _serde::export::Some(
                                    match _serde::de::MapAccess::next_value::<Option<bool>>(
                                        &mut __map,
                                    ) {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    },
                                );
                            }
                            __Field::__field4 => {
                                if _serde::export::Option::is_some(&__field4) {
                                    return _serde::export::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("test4"),
                                    );
                                }
                                __field4 = _serde::export::Some(
                                    match _serde::de::MapAccess::next_value::<Option<Vec<i32>>>(
                                        &mut __map,
                                    ) {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    },
                                );
                            }
                            _ => {
                                let _ = match _serde::de::MapAccess::next_value::<
                                    _serde::de::IgnoredAny,
                                >(&mut __map)
                                {
                                    _serde::export::Ok(__val) => __val,
                                    _serde::export::Err(__err) => {
                                        return _serde::export::Err(__err);
                                    }
                                };
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::export::Some(__field0) => __field0,
                        _serde::export::None => match _serde::private::de::missing_field("name") {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        },
                    };
                    let __field1 = match __field1 {
                        _serde::export::Some(__field1) => __field1,
                        _serde::export::None => match _serde::private::de::missing_field("test1") {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        },
                    };
                    let __field2 = match __field2 {
                        _serde::export::Some(__field2) => __field2,
                        _serde::export::None => match _serde::private::de::missing_field("test2") {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        },
                    };
                    let __field3 = match __field3 {
                        _serde::export::Some(__field3) => __field3,
                        _serde::export::None => match _serde::private::de::missing_field("test3") {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        },
                    };
                    let __field4 = match __field4 {
                        _serde::export::Some(__field4) => __field4,
                        _serde::export::None => match _serde::private::de::missing_field("test4") {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        },
                    };
                    _serde::export::Ok(InsertClub {
                        name: __field0,
                        test1: __field1,
                        test2: __field2,
                        test3: __field3,
                        test4: __field4,
                    })
                }
            }
            const FIELDS: &'static [&'static str] = &["name", "test1", "test2", "test3", "test4"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "InsertClub",
                FIELDS,
                __Visitor {
                    marker: _serde::export::PhantomData::<InsertClub>,
                    lifetime: _serde::export::PhantomData,
                },
            )
        }
    }
};
impl InsertClub {
    /// Insert a row into the database.
    async fn insert(self, __con: &mut sqlx::PgConnection) -> sqlx::Result<Club> {
        use sqlx::Connection;
        let mut tx = __con.begin().await?;
        let rec = { { use sqlx :: Arguments as _ ; let arg0 = & ( self . name ) ; let arg1 = & ( self . test1 ) ; let arg2 = & ( enum_convert ( & self . test2 ) ) ; let arg3 = & ( self . test3 ) ; let arg4 = & ( my_convert ( & self . test4 ) ) ; if false { use sqlx :: ty_match :: { WrapSameExt as _ , MatchBorrowExt as _ } ; let _expr = sqlx :: ty_match :: dupe_value ( arg0 ) ; let ty_check = sqlx :: ty_match :: WrapSame :: < & str , _ > :: new ( & _expr ) . wrap_same ( ) ; let ( mut _ty_check , match_borrow ) = sqlx :: ty_match :: MatchBorrow :: new ( ty_check , & _expr ) ; _ty_check = match_borrow . match_borrow ( ) ; { { :: std :: rt :: begin_panic ( "explicit panic" ) } } ; } if false { use sqlx :: ty_match :: { WrapSameExt as _ , MatchBorrowExt as _ } ; let _expr = sqlx :: ty_match :: dupe_value ( arg1 ) ; let ty_check = sqlx :: ty_match :: WrapSame :: < & str , _ > :: new ( & _expr ) . wrap_same ( ) ; let ( mut _ty_check , match_borrow ) = sqlx :: ty_match :: MatchBorrow :: new ( ty_check , & _expr ) ; _ty_check = match_borrow . match_borrow ( ) ; { { :: std :: rt :: begin_panic ( "explicit panic" ) } } ; } if false { use sqlx :: ty_match :: { WrapSameExt as _ , MatchBorrowExt as _ } ; let _expr = sqlx :: ty_match :: dupe_value ( arg2 ) ; let ty_check = sqlx :: ty_match :: WrapSame :: < i32 , _ > :: new ( & _expr ) . wrap_same ( ) ; let ( mut _ty_check , match_borrow ) = sqlx :: ty_match :: MatchBorrow :: new ( ty_check , & _expr ) ; _ty_check = match_borrow . match_borrow ( ) ; { { :: std :: rt :: begin_panic ( "explicit panic" ) } } ; } if false { use sqlx :: ty_match :: { WrapSameExt as _ , MatchBorrowExt as _ } ; let _expr = sqlx :: ty_match :: dupe_value ( arg3 ) ; let ty_check = sqlx :: ty_match :: WrapSame :: < bool , _ > :: new ( & _expr ) . wrap_same ( ) ; let ( mut _ty_check , match_borrow ) = sqlx :: ty_match :: MatchBorrow :: new ( ty_check , & _expr ) ; _ty_check = match_borrow . match_borrow ( ) ; { { :: std :: rt :: begin_panic ( "explicit panic" ) } } ; } if false { use sqlx :: ty_match :: { WrapSameExt as _ , MatchBorrowExt as _ } ; let _expr = sqlx :: ty_match :: dupe_value ( arg4 ) ; let ty_check = sqlx :: ty_match :: WrapSame :: < & [ i32 ] , _ > :: new ( & _expr ) . wrap_same ( ) ; let ( mut _ty_check , match_borrow ) = sqlx :: ty_match :: MatchBorrow :: new ( ty_check , & _expr ) ; _ty_check = match_borrow . match_borrow ( ) ; { { :: std :: rt :: begin_panic ( "explicit panic" ) } } ; } let mut query_args = < sqlx :: postgres :: Postgres as sqlx :: database :: HasArguments > :: default ( ) ; query_args . reserve ( 5usize , 0 + sqlx :: encode :: Encode :: < sqlx :: postgres :: Postgres > :: size_hint ( arg0 ) + sqlx :: encode :: Encode :: < sqlx :: postgres :: Postgres > :: size_hint ( arg1 ) + sqlx :: encode :: Encode :: < sqlx :: postgres :: Postgres > :: size_hint ( arg2 ) + sqlx :: encode :: Encode :: < sqlx :: postgres :: Postgres > :: size_hint ( arg3 ) + sqlx :: encode :: Encode :: < sqlx :: postgres :: Postgres > :: size_hint ( arg4 ) ) ; query_args . add ( arg0 ) ; query_args . add ( arg1 ) ; query_args . add ( arg2 ) ; query_args . add ( arg3 ) ; query_args . add ( arg4 ) ; struct Record { id : i32 , } # [ automatically_derived ] # [ allow ( unused_qualifications ) ] impl :: core :: fmt :: Debug for Record { fn fmt ( & self , f : & mut :: core :: fmt :: Formatter ) -> :: core :: fmt :: Result { match * self { Record { id : ref __self_0_0 } => { let mut debug_trait_builder = f . debug_struct ( "Record" ) ; let _ = debug_trait_builder . field ( "id" , & & ( * __self_0_0 ) ) ; debug_trait_builder . finish ( ) } } } } sqlx :: query_with :: < sqlx :: postgres :: Postgres , _ > ( "INSERT INTO clubs (name,test1,test2,test3,test4) VALUES ($1,$2,$3,$4,$5) RETURNING id" , query_args ) . try_map ( | row : sqlx :: postgres :: PgRow | { use sqlx :: Row as _ ; let id = row . try_get_unchecked :: < i32 , _ > ( 0usize ) ? ; Ok ( Record { id : id , } ) } ) } } . fetch_one ( & mut tx ) . await ? ;
        tx.commit().await?;
        Ok(Club {
            id: rec.id as _,
            name: self.name,
            test1: self.test1,
            test2: self.test2,
            test3: self.test3,
            test4: self.test4,
        })
    }
}
struct PatchClub {
    name: Option<String>,
    test1: Option<String>,
    test2: Option<TestEnum>,
    test3: Option<Option<bool>>,
    test4: Option<Option<Vec<i32>>>,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::default::Default for PatchClub {
    #[inline]
    fn default() -> PatchClub {
        PatchClub {
            name: ::core::default::Default::default(),
            test1: ::core::default::Default::default(),
            test2: ::core::default::Default::default(),
            test3: ::core::default::Default::default(),
            test4: ::core::default::Default::default(),
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::clone::Clone for PatchClub {
    #[inline]
    fn clone(&self) -> PatchClub {
        match *self {
            PatchClub {
                name: ref __self_0_0,
                test1: ref __self_0_1,
                test2: ref __self_0_2,
                test3: ref __self_0_3,
                test4: ref __self_0_4,
            } => PatchClub {
                name: ::core::clone::Clone::clone(&(*__self_0_0)),
                test1: ::core::clone::Clone::clone(&(*__self_0_1)),
                test2: ::core::clone::Clone::clone(&(*__self_0_2)),
                test3: ::core::clone::Clone::clone(&(*__self_0_3)),
                test4: ::core::clone::Clone::clone(&(*__self_0_4)),
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for PatchClub {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            PatchClub {
                name: ref __self_0_0,
                test1: ref __self_0_1,
                test2: ref __self_0_2,
                test3: ref __self_0_3,
                test4: ref __self_0_4,
            } => {
                let mut debug_trait_builder = f.debug_struct("PatchClub");
                let _ = debug_trait_builder.field("name", &&(*__self_0_0));
                let _ = debug_trait_builder.field("test1", &&(*__self_0_1));
                let _ = debug_trait_builder.field("test2", &&(*__self_0_2));
                let _ = debug_trait_builder.field("test3", &&(*__self_0_3));
                let _ = debug_trait_builder.field("test4", &&(*__self_0_4));
                debug_trait_builder.finish()
            }
        }
    }
}
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(rust_2018_idioms, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for PatchClub {
        fn serialize<__S>(&self, __serializer: __S) -> _serde::export::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = match _serde::Serializer::serialize_struct(
                __serializer,
                "PatchClub",
                false as usize + 1 + 1 + 1 + 1 + 1,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "name",
                &self.name,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "test1",
                &self.test1,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "test2",
                &self.test2,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "test3",
                &self.test3,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "test4",
                &self.test4,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(rust_2018_idioms, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for PatchClub {
        fn deserialize<__D>(__deserializer: __D) -> _serde::export::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            enum __Field {
                __field0,
                __field1,
                __field2,
                __field3,
                __field4,
                __ignore,
            }
            struct __FieldVisitor;
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::export::Formatter,
                ) -> _serde::export::fmt::Result {
                    _serde::export::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::export::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::export::Ok(__Field::__field0),
                        1u64 => _serde::export::Ok(__Field::__field1),
                        2u64 => _serde::export::Ok(__Field::__field2),
                        3u64 => _serde::export::Ok(__Field::__field3),
                        4u64 => _serde::export::Ok(__Field::__field4),
                        _ => _serde::export::Err(_serde::de::Error::invalid_value(
                            _serde::de::Unexpected::Unsigned(__value),
                            &"field index 0 <= i < 5",
                        )),
                    }
                }
                fn visit_str<__E>(self, __value: &str) -> _serde::export::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "name" => _serde::export::Ok(__Field::__field0),
                        "test1" => _serde::export::Ok(__Field::__field1),
                        "test2" => _serde::export::Ok(__Field::__field2),
                        "test3" => _serde::export::Ok(__Field::__field3),
                        "test4" => _serde::export::Ok(__Field::__field4),
                        _ => _serde::export::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::export::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"name" => _serde::export::Ok(__Field::__field0),
                        b"test1" => _serde::export::Ok(__Field::__field1),
                        b"test2" => _serde::export::Ok(__Field::__field2),
                        b"test3" => _serde::export::Ok(__Field::__field3),
                        b"test4" => _serde::export::Ok(__Field::__field4),
                        _ => _serde::export::Ok(__Field::__ignore),
                    }
                }
            }
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(__deserializer: __D) -> _serde::export::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            struct __Visitor<'de> {
                marker: _serde::export::PhantomData<PatchClub>,
                lifetime: _serde::export::PhantomData<&'de ()>,
            }
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = PatchClub;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::export::Formatter,
                ) -> _serde::export::fmt::Result {
                    _serde::export::Formatter::write_str(__formatter, "struct PatchClub")
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::export::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match match _serde::de::SeqAccess::next_element::<Option<String>>(
                        &mut __seq,
                    ) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    } {
                        _serde::export::Some(__value) => __value,
                        _serde::export::None => {
                            return _serde::export::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"struct PatchClub with 5 elements",
                            ));
                        }
                    };
                    let __field1 = match match _serde::de::SeqAccess::next_element::<Option<String>>(
                        &mut __seq,
                    ) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    } {
                        _serde::export::Some(__value) => __value,
                        _serde::export::None => {
                            return _serde::export::Err(_serde::de::Error::invalid_length(
                                1usize,
                                &"struct PatchClub with 5 elements",
                            ));
                        }
                    };
                    let __field2 = match match _serde::de::SeqAccess::next_element::<Option<TestEnum>>(
                        &mut __seq,
                    ) {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    } {
                        _serde::export::Some(__value) => __value,
                        _serde::export::None => {
                            return _serde::export::Err(_serde::de::Error::invalid_length(
                                2usize,
                                &"struct PatchClub with 5 elements",
                            ));
                        }
                    };
                    let __field3 = match match _serde::de::SeqAccess::next_element::<
                        Option<Option<bool>>,
                    >(&mut __seq)
                    {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    } {
                        _serde::export::Some(__value) => __value,
                        _serde::export::None => {
                            return _serde::export::Err(_serde::de::Error::invalid_length(
                                3usize,
                                &"struct PatchClub with 5 elements",
                            ));
                        }
                    };
                    let __field4 = match match _serde::de::SeqAccess::next_element::<
                        Option<Option<Vec<i32>>>,
                    >(&mut __seq)
                    {
                        _serde::export::Ok(__val) => __val,
                        _serde::export::Err(__err) => {
                            return _serde::export::Err(__err);
                        }
                    } {
                        _serde::export::Some(__value) => __value,
                        _serde::export::None => {
                            return _serde::export::Err(_serde::de::Error::invalid_length(
                                4usize,
                                &"struct PatchClub with 5 elements",
                            ));
                        }
                    };
                    _serde::export::Ok(PatchClub {
                        name: __field0,
                        test1: __field1,
                        test2: __field2,
                        test3: __field3,
                        test4: __field4,
                    })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::export::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::export::Option<Option<String>> = _serde::export::None;
                    let mut __field1: _serde::export::Option<Option<String>> = _serde::export::None;
                    let mut __field2: _serde::export::Option<Option<TestEnum>> =
                        _serde::export::None;
                    let mut __field3: _serde::export::Option<Option<Option<bool>>> =
                        _serde::export::None;
                    let mut __field4: _serde::export::Option<Option<Option<Vec<i32>>>> =
                        _serde::export::None;
                    while let _serde::export::Some(__key) =
                        match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        }
                    {
                        match __key {
                            __Field::__field0 => {
                                if _serde::export::Option::is_some(&__field0) {
                                    return _serde::export::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("name"),
                                    );
                                }
                                __field0 = _serde::export::Some(
                                    match _serde::de::MapAccess::next_value::<Option<String>>(
                                        &mut __map,
                                    ) {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    },
                                );
                            }
                            __Field::__field1 => {
                                if _serde::export::Option::is_some(&__field1) {
                                    return _serde::export::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("test1"),
                                    );
                                }
                                __field1 = _serde::export::Some(
                                    match _serde::de::MapAccess::next_value::<Option<String>>(
                                        &mut __map,
                                    ) {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    },
                                );
                            }
                            __Field::__field2 => {
                                if _serde::export::Option::is_some(&__field2) {
                                    return _serde::export::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("test2"),
                                    );
                                }
                                __field2 = _serde::export::Some(
                                    match _serde::de::MapAccess::next_value::<Option<TestEnum>>(
                                        &mut __map,
                                    ) {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    },
                                );
                            }
                            __Field::__field3 => {
                                if _serde::export::Option::is_some(&__field3) {
                                    return _serde::export::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("test3"),
                                    );
                                }
                                __field3 = _serde::export::Some(
                                    match _serde::de::MapAccess::next_value::<Option<Option<bool>>>(
                                        &mut __map,
                                    ) {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    },
                                );
                            }
                            __Field::__field4 => {
                                if _serde::export::Option::is_some(&__field4) {
                                    return _serde::export::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("test4"),
                                    );
                                }
                                __field4 = _serde::export::Some(
                                    match _serde::de::MapAccess::next_value::<
                                        Option<Option<Vec<i32>>>,
                                    >(&mut __map)
                                    {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    },
                                );
                            }
                            _ => {
                                let _ = match _serde::de::MapAccess::next_value::<
                                    _serde::de::IgnoredAny,
                                >(&mut __map)
                                {
                                    _serde::export::Ok(__val) => __val,
                                    _serde::export::Err(__err) => {
                                        return _serde::export::Err(__err);
                                    }
                                };
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::export::Some(__field0) => __field0,
                        _serde::export::None => match _serde::private::de::missing_field("name") {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        },
                    };
                    let __field1 = match __field1 {
                        _serde::export::Some(__field1) => __field1,
                        _serde::export::None => match _serde::private::de::missing_field("test1") {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        },
                    };
                    let __field2 = match __field2 {
                        _serde::export::Some(__field2) => __field2,
                        _serde::export::None => match _serde::private::de::missing_field("test2") {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        },
                    };
                    let __field3 = match __field3 {
                        _serde::export::Some(__field3) => __field3,
                        _serde::export::None => match _serde::private::de::missing_field("test3") {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        },
                    };
                    let __field4 = match __field4 {
                        _serde::export::Some(__field4) => __field4,
                        _serde::export::None => match _serde::private::de::missing_field("test4") {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        },
                    };
                    _serde::export::Ok(PatchClub {
                        name: __field0,
                        test1: __field1,
                        test2: __field2,
                        test3: __field3,
                        test4: __field4,
                    })
                }
            }
            const FIELDS: &'static [&'static str] = &["name", "test1", "test2", "test3", "test4"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "PatchClub",
                FIELDS,
                __Visitor {
                    marker: _serde::export::PhantomData::<PatchClub>,
                    lifetime: _serde::export::PhantomData,
                },
            )
        }
    }
};
impl PatchClub {
    fn set_name(mut self, value: String) -> Self {
        self.name = Some(value);
        self
    }
    fn set_test1(mut self, value: String) -> Self {
        self.test1 = Some(value);
        self
    }
    fn set_test2(mut self, value: TestEnum) -> Self {
        self.test2 = Some(value);
        self
    }
    fn set_test3(mut self, value: Option<bool>) -> Self {
        self.test3 = Some(value);
        self
    }
    fn set_test4(mut self, value: Option<Vec<i32>>) -> Self {
        self.test4 = Some(value);
        self
    }
}
impl PatchClub {
    async fn patch(
        &self,
        con: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
        id: &i32,
    ) -> sqlx::Result<()> {
        let mut columns = ::alloc::vec::Vec::new();
        let mut count = 2;
        if self.name.is_some() {
            columns.push({
                let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                    &["", " = $"],
                    &match (&"name", &count) {
                        (arg0, arg1) => [
                            ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                            ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                        ],
                    },
                ));
                res
            });
            count += 1;
        }
        if self.test1.is_some() {
            columns.push({
                let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                    &["", " = $"],
                    &match (&"test1", &count) {
                        (arg0, arg1) => [
                            ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                            ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                        ],
                    },
                ));
                res
            });
            count += 1;
        }
        if self.test2.is_some() {
            columns.push({
                let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                    &["", " = $"],
                    &match (&"test2", &count) {
                        (arg0, arg1) => [
                            ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                            ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                        ],
                    },
                ));
                res
            });
            count += 1;
        }
        if self.test3.is_some() {
            columns.push({
                let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                    &["", " = $"],
                    &match (&"test3", &count) {
                        (arg0, arg1) => [
                            ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                            ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                        ],
                    },
                ));
                res
            });
            count += 1;
        }
        if self.test4.is_some() {
            columns.push({
                let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                    &["", " = $"],
                    &match (&"test4", &count) {
                        (arg0, arg1) => [
                            ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                            ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                        ],
                    },
                ));
                res
            });
            count += 1;
        }
        let columns = columns.join(", ");
        let sql = {
            let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                &["UPDATE ", " SET ", " WHERE id = $1"],
                &match (&"clubs", &columns) {
                    (arg0, arg1) => [
                        ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                        ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                    ],
                },
            ));
            res
        };
        let mut query = sqlx::query(&sql).bind(id);
        if let Some(value) = self.name.as_ref() {
            query = query.bind(value)
        }
        if let Some(value) = self.test1.as_ref() {
            query = query.bind(value)
        }
        if let Some(value) = self.test2.as_ref() {
            query = query.bind(enum_convert(value))
        }
        if let Some(value) = self.test3.as_ref() {
            query = query.bind(value)
        }
        if let Some(value) = self.test4.as_ref() {
            query = query.bind(my_convert(value))
        }
        query.execute(con).await?;
        Ok(())
    }
}
impl Club {
    async fn patch(
        &mut self,
        con: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
        update: PatchClub,
    ) -> sqlx::Result<()> {
        PatchClub::patch(&update, con, &self.id).await?;
        if let Some(new_value) = update.name {
            self.name = new_value;
        }
        if let Some(new_value) = update.test1 {
            self.test1 = new_value;
        }
        if let Some(new_value) = update.test2 {
            self.test2 = new_value;
        }
        if let Some(new_value) = update.test3 {
            self.test3 = new_value;
        }
        if let Some(new_value) = update.test4 {
            self.test4 = new_value;
        }
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for Club {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            Club {
                id: ref __self_0_0,
                name: ref __self_0_1,
                test1: ref __self_0_2,
                test2: ref __self_0_3,
                test3: ref __self_0_4,
                test4: ref __self_0_5,
            } => {
                let mut debug_trait_builder = f.debug_struct("Club");
                let _ = debug_trait_builder.field("id", &&(*__self_0_0));
                let _ = debug_trait_builder.field("name", &&(*__self_0_1));
                let _ = debug_trait_builder.field("test1", &&(*__self_0_2));
                let _ = debug_trait_builder.field("test2", &&(*__self_0_3));
                let _ = debug_trait_builder.field("test3", &&(*__self_0_4));
                let _ = debug_trait_builder.field("test4", &&(*__self_0_5));
                debug_trait_builder.finish()
            }
        }
    }
}
#[allow(unused)]
fn my_convert(t: &Option<Vec<i32>>) -> Option<&[i32]> {
    t.as_ref().map(|the_t| the_t.as_slice())
}
