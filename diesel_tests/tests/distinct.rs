use super::schema::*;
use diesel::*;

#[test]
fn simple_distinct() {
    use crate::schema::users::dsl::*;

    let connection = &mut connection();
    diesel::sql_query("INSERT INTO users (name) VALUES ('Sean'), ('Sean'), ('Tess')")
        .execute(connection)
        .unwrap();

    let source = users.select(name).distinct().order(name);
    let expected_data = vec!["Sean".to_string(), "Tess".to_string()];
    let data: Vec<String> = source.load(connection).unwrap();

    assert_eq!(expected_data, data);
}

#[cfg(feature = "postgres")]
#[test]
fn distinct_on() {
    use crate::schema::users::dsl::*;

    let connection = &mut connection();
    diesel::sql_query(
            "INSERT INTO users (name, hair_color) VALUES ('Sean', 'black'), ('Sean', NULL), ('Tess', NULL), ('Tess', NULL)",
        ).execute(connection)
        .unwrap();

    let source = users
        .select((name, hair_color))
        .order(name)
        .distinct_on(name);
    let mut expected_data = vec![
        ("Sean".to_string(), Some("black".to_string())),
        ("Tess".to_string(), None),
    ];
    let data: Vec<_> = source.load(connection).unwrap();

    assert_eq!(expected_data, data);

    let source = users
        .select((name, hair_color))
        .order(name.asc())
        .distinct_on(name);
    let data: Vec<_> = source.load(connection).unwrap();

    assert_eq!(expected_data, data);

    let source = users
        .select((name, hair_color))
        .order(name.desc())
        .distinct_on(name);
    let data: Vec<_> = source.load(connection).unwrap();

    expected_data.reverse();
    assert_eq!(expected_data, data);
}

#[cfg(feature = "postgres")]
#[test]
fn distinct_on_select_by() {
    use crate::schema::users::dsl::*;

    let connection = &mut connection();
    diesel::sql_query(
            "INSERT INTO users (name, hair_color) VALUES ('Sean', 'black'), ('Sean', NULL), ('Tess', NULL), ('Tess', NULL)",
        ).execute(connection)
        .unwrap();

    let source = users
        .select(NewUser::as_select())
        .order(name)
        .distinct_on(name);
    let expected_data = vec![
        NewUser::new("Sean", Some("black")),
        NewUser::new("Tess", None),
    ];
    let data: Vec<_> = source.load(connection).unwrap();

    assert_eq!(expected_data, data);
}

#[cfg(feature = "postgres")]
#[test]
fn distinct_on_select_order_by_two_columns() {
    use crate::schema::users::dsl::*;

    let connection = &mut connection();
    diesel::sql_query(
            "INSERT INTO users (name, hair_color) VALUES ('Sean', 'black'), ('Sean', 'aqua'), ('Tess', 'bronze'), ('Tess', 'champagne')",
        ).execute(connection)
        .unwrap();

    let source = users
        .select((name, hair_color))
        .order((name, hair_color.desc()))
        .distinct_on(name);
    let expected_data = vec![
        NewUser::new("Sean", Some("black")),
        NewUser::new("Tess", Some("champagne")),
    ];
    let data: Vec<_> = source.load(connection).unwrap();

    assert_eq!(expected_data, data);

    let source = users
        .select((name, hair_color))
        .order((name.asc(), hair_color.desc()))
        .distinct_on(name);
    let data: Vec<_> = source.load(connection).unwrap();

    assert_eq!(expected_data, data);

    let source = users
        .select((name, hair_color))
        .order((name.desc(), hair_color.desc()))
        .distinct_on(name);
    let expected_data = vec![
        NewUser::new("Tess", Some("champagne")),
        NewUser::new("Sean", Some("black")),
    ];
    let data: Vec<_> = source.load(connection).unwrap();

    assert_eq!(expected_data, data);

    let source = users
        .select((name, hair_color))
        .order((name.desc(), hair_color))
        .distinct_on(name);
    let expected_data = vec![
        NewUser::new("Tess", Some("bronze")),
        NewUser::new("Sean", Some("aqua")),
    ];
    let data: Vec<_> = source.load(connection).unwrap();

    assert_eq!(expected_data, data);
}

#[cfg(feature = "postgres")]
#[test]
fn distinct_of_multiple_columns() {
    use crate::schema::posts;
    use crate::schema::users;

    let mut connection = connection_with_sean_and_tess_in_users_table();

    let sean = find_user_by_name("Sean", &mut connection);
    let tess = find_user_by_name("Tess", &mut connection);

    let new_posts = vec![
        NewPost::new(sean.id, "1", Some("1")),
        NewPost::new(sean.id, "2", Some("2")),
        NewPost::new(sean.id, "3", Some("1")),
        NewPost::new(sean.id, "4", Some("2")),
        NewPost::new(tess.id, "5", Some("1")),
        NewPost::new(tess.id, "6", Some("2")),
        NewPost::new(tess.id, "7", Some("1")),
        NewPost::new(tess.id, "8", Some("2")),
    ];
    insert_into(posts::table)
        .values(&new_posts)
        .execute(&mut connection)
        .unwrap();
    let posts = posts::table
        .order(posts::id)
        .load::<Post>(&mut connection)
        .unwrap();

    let data = posts::table
        .inner_join(users::table)
        .order((users::id, posts::body, posts::title))
        .distinct_on((users::id, posts::body))
        .load(&mut connection);
    let expected = vec![
        ((posts[0].clone(), sean.clone())),
        ((posts[1].clone(), sean.clone())),
        ((posts[4].clone(), tess.clone())),
        ((posts[5].clone(), tess.clone())),
    ];

    assert_eq!(Ok(expected), data);
}

// #[cfg(feature = "postgres")]
// #[test]
// #[should_panic(expected = "TBD")]
// fn invalid_distinct_on_or_order_detected() {
//     use crate::schema::users;
//     use crate::schema::posts;
//
//     let mut connection = connection();
//
//     posts::table
//         .inner_join(users::table)
//         .order((users::id, posts::title))
//         .distinct_on((users::id, posts::body))
//         .load(&mut connection);
// }
