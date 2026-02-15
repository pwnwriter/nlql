// basic sql safety checks
// catches obvious dangerous stuff but not everything

pub struct Safety {
    pub is_dangerous: bool,
    pub reason: String,
    pub warning: Option<String>,
}

impl Safety {
    pub fn check(sql: &str) -> Self {
        let sql_upper = sql.to_uppercase();

        // these are almost always bad news
        let dangerous = [
            ("DROP ", "DROP can permanently delete tables"),
            ("TRUNCATE ", "TRUNCATE deletes all data"),
            ("ALTER ", "ALTER modifies table structure"),
            ("; DROP", "looks like sql injection"),
            ("--", "sql comment, possible injection"),
        ];

        for (pattern, reason) in dangerous {
            if sql_upper.contains(pattern) {
                return Self {
                    is_dangerous: true,
                    reason: reason.to_string(),
                    warning: None,
                };
            }
        }

        // delete/update without where = wipe everything
        if sql_upper.contains("DELETE") && !sql_upper.contains("WHERE") {
            return Self {
                is_dangerous: true,
                reason: "DELETE without WHERE deletes all rows".to_string(),
                warning: None,
            };
        }

        if sql_upper.contains("UPDATE") && !sql_upper.contains("WHERE") {
            return Self {
                is_dangerous: true,
                reason: "UPDATE without WHERE updates all rows".to_string(),
                warning: None,
            };
        }

        // not dangerous but worth mentioning
        let warning = if sql_upper.contains("DELETE") {
            Some("this will delete data".to_string())
        } else if sql_upper.contains("UPDATE") {
            Some("this will update data".to_string())
        } else if sql_upper.contains("INSERT") {
            Some("this will insert data".to_string())
        } else {
            None
        };

        Self {
            is_dangerous: false,
            reason: String::new(),
            warning,
        }
    }
}
