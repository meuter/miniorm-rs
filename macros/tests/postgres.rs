mod table_name {

    #[test]
    fn default_name() {
        #[derive(miniorm::Schema)]
        struct Point {
            #[postgres(INTEGER NOT NULL)]
            x: i64,
            #[postgres(INTEGER NOT NULL)]
            y: i64,
        }

        assert_eq!(
            <Point as miniorm::traits::Schema<sqlx::Postgres>>::TABLE_NAME,
            "point"
        );
    }

    #[test]
    fn rename() {
        #[derive(miniorm::Schema)]
        #[sqlx(rename = "coord")]
        struct Point {
            #[postgres(INTEGER NOT NULL)]
            x: i64,
            #[postgres(INTEGER NOT NULL)]
            y: i64,
        }

        assert_eq!(
            <Point as miniorm::traits::Schema<sqlx::Postgres>>::TABLE_NAME,
            "coord"
        );
    }
}
