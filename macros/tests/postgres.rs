use miniorm::prelude::*;
use sqlx::Postgres;

mod table_name {
    use super::*;

    #[test]
    fn default() {
        #[derive(Entity)]
        struct Point {
            #[postgres(INTEGER NOT NULL)]
            x: i64,
            #[postgres(INTEGER NOT NULL)]
            y: i64,
        }

        assert_eq!(<Point as Schema<Postgres>>::MINIORM_TABLE_NAME, "point");
    }

    #[test]
    fn rename() {
        #[derive(Entity)]
        #[sqlx(rename = "coord")]
        struct Point {
            #[postgres(INTEGER NOT NULL)]
            x: i64,
            #[postgres(INTEGER NOT NULL)]
            y: i64,
        }

        assert_eq!(<Point as Schema<Postgres>>::MINIORM_TABLE_NAME, "coord");
    }
}

mod columns {
    use super::*;

    #[test]
    fn default() {
        #[derive(Entity)]
        struct Point {
            #[postgres(XXX YYY)]
            x: i64,
            #[postgres(AAA BBB)]
            y: i64,
        }

        let columns = <Point as Schema<Postgres>>::MINIORM_COLUMNS;

        assert_eq!(columns.len(), 2);
        assert_eq!(columns[0], "x");
        assert_eq!(columns[1], "y");
    }

    #[test]
    fn skip() {
        #[derive(Entity)]
        struct Point {
            #[postgres(XXX YYY)]
            x: i64,
            #[sqlx(skip)]
            #[allow(unused)]
            y: i64,
        }

        let columns = <Point as Schema<Postgres>>::MINIORM_COLUMNS;

        assert_eq!(columns.len(), 1);
        assert_eq!(columns[0], "x");
    }

    #[test]
    fn rename() {
        #[derive(Entity)]
        struct Point {
            #[postgres(XXX YYY)]
            x: i64,
            #[postgres(AAA BBB)]
            #[sqlx(rename = "z")]
            y: i64,
        }

        let columns = <Point as Schema<Postgres>>::MINIORM_COLUMNS;

        assert_eq!(columns.len(), 2);
        assert_eq!(columns[0], "x");
        assert_eq!(columns[1], "z");
    }
}
