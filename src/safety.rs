pub struct Safety {
    pub is_dangerous: bool,
    pub reason: String,
    pub warning: Option<String>,
}

impl Safety {
    pub fn check(sql: &str) -> Self {
        let sql_upper = sql.to_uppercase();

        // Dangerous patterns
        let dangerous_patterns = [
            ("DROP ", "DROP statement can permanently delete tables/databases"),
            ("TRUNCATE ", "TRUNCATE will delete all data from the table"),
            ("ALTER ", "ALTER can modify table structure"),
            ("; DROP", "Possible SQL injection pattern detected"),
            ("--", "SQL comment detected, possible injection"),
        ];

        for (pattern, reason) in dangerous_patterns {
            if sql_upper.contains(pattern) {
                return Self {
                    is_dangerous: true,
                    reason: reason.to_string(),
                    warning: None,
                };
            }
        }

        // DELETE without WHERE is dangerous
        if sql_upper.contains("DELETE") && !sql_upper.contains("WHERE") {
            return Self {
                is_dangerous: true,
                reason: "DELETE without WHERE clause will delete all rows".to_string(),
                warning: None,
            };
        }

        // UPDATE without WHERE is dangerous
        if sql_upper.contains("UPDATE") && !sql_upper.contains("WHERE") {
            return Self {
                is_dangerous: true,
                reason: "UPDATE without WHERE clause will update all rows".to_string(),
                warning: None,
            };
        }

        // Warnings (not blocking)
        let mut warning = None;

        if sql_upper.contains("DELETE") {
            warning = Some("This query will DELETE data".to_string());
        } else if sql_upper.contains("UPDATE") {
            warning = Some("This query will UPDATE data".to_string());
        } else if sql_upper.contains("INSERT") {
            warning = Some("This query will INSERT data".to_string());
        }

        Self {
            is_dangerous: false,
            reason: String::new(),
            warning,
        }
    }
}
