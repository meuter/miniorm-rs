use miniorm::{Entity, Schema};
use sqlx::MySql;

mod table_name {
    use super::*;

    #[test]
    fn default() {
        #[derive(Entity)]
        struct Point {
            #[mysql(INTEGER NOT NULL)]
            x: i64,
            #[mysql(INTEGER NOT NULL)]
            y: i64,
        }

        assert_eq!(<Point as Schema<MySql>>::TABLE_NAME, "point");
    }

    #[test]
    fn rename() {
        #[derive(Entity)]
        #[sqlx(rename = "coord")]
        struct Point {
            #[mysql(INTEGER NOT NULL)]
            x: i64,
            #[mysql(INTEGER NOT NULL)]
            y: i64,
        }

        assert_eq!(<Point as Schema<MySql>>::TABLE_NAME, "coord");
    }
}

mod columns {
    use super::*;

    #[test]
    fn default() {
        #[derive(Entity)]
        struct Point {
            #[mysql(XXX YYY)]
            x: i64,
            #[mysql(AAA BBB)]
            y: i64,
        }

        let columns = <Point as Schema<MySql>>::COLUMNS;

        assert_eq!(columns.len(), 2);
        assert_eq!(columns[0].0, "x");
        assert_eq!(columns[0].1, "XXX YYY");
        assert_eq!(columns[1].0, "y");
        assert_eq!(columns[1].1, "AAA BBB");
    }

    #[test]
    fn skip() {
        #[derive(Entity)]
        struct Point {
            #[mysql(XXX YYY)]
            x: i64,
            #[sqlx(skip)]
            #[allow(unused)]
            y: i64,
        }

        let columns = <Point as Schema<MySql>>::COLUMNS;

        assert_eq!(columns.len(), 1);
        assert_eq!(columns[0].0, "x");
        assert_eq!(columns[0].1, "XXX YYY");
    }

    #[test]
    fn rename() {
        #[derive(Entity)]
        struct Point {
            #[mysql(XXX YYY)]
            x: i64,
            #[mysql(AAA BBB)]
            #[sqlx(rename = "z")]
            y: i64,
        }

        let columns = <Point as Schema<MySql>>::COLUMNS;

        assert_eq!(columns.len(), 2);
        assert_eq!(columns[0].0, "x");
        assert_eq!(columns[0].1, "XXX YYY");
        assert_eq!(columns[1].0, "z");
        assert_eq!(columns[1].1, "AAA BBB");
    }
}
