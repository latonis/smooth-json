use serde_json::json;
use smooth_json::Flattener;

#[test]
fn basic_user_profile() {
    let flattener = Flattener::new();

    let user_profile = json!({
        "id": 12345,
        "username": "johndoe",
        "email": "john@example.com",
        "profile": {
            "first_name": "John",
            "last_name": "Doe",
            "age": 30
        },
        "preferences": {
            "theme": "dark",
            "notifications": {
                "email": true,
                "push": false
            }
        }
    });

    let flattened = flattener.flatten(&user_profile);

    assert_eq!(flattened["id"], 12345);
    assert_eq!(flattened["username"], "johndoe");
    assert_eq!(flattened["email"], "john@example.com");
    assert_eq!(flattened["profile.first_name"], "John");
    assert_eq!(flattened["profile.last_name"], "Doe");
    assert_eq!(flattened["profile.age"], 30);
    assert_eq!(flattened["preferences.theme"], "dark");
    assert_eq!(flattened["preferences.notifications.email"], true);
    assert_eq!(flattened["preferences.notifications.push"], false);
}

#[test]
fn github_api_issue_response() {
    let flattener = Flattener::new();

    let issue = json!({
        "id": 1,
        "number": 42,
        "title": "Found a bug",
        "state": "open",
        "user": {
            "login": "octocat",
            "id": 1,
            "avatar_url": "https://github.com/images/error/octocat_happy.gif"
        },
        "labels": [
            {"id": 1, "name": "bug", "color": "d73a4a"},
            {"id": 2, "name": "enhancement", "color": "a2eeef"}
        ],
        "assignees": []
    });

    let flattened = flattener.flatten(&issue);

    assert_eq!(flattened["id"], 1);
    assert_eq!(flattened["number"], 42);
    assert_eq!(flattened["title"], "Found a bug");
    assert_eq!(flattened["state"], "open");
    assert_eq!(flattened["user.login"], "octocat");
    assert_eq!(flattened["user.id"], 1);
    assert_eq!(
        flattened["user.avatar_url"],
        "https://github.com/images/error/octocat_happy.gif"
    );

    // Labels should be flattened into arrays
    assert!(flattened["labels.id"].is_array());
    assert_eq!(flattened["labels.id"][0], 1);
    assert_eq!(flattened["labels.id"][1], 2);
    assert_eq!(flattened["labels.name"][0], "bug");
    assert_eq!(flattened["labels.name"][1], "enhancement");

    // Empty arrays should be preserved when flattening
    assert!(flattened["assignees"].is_array());
    assert_eq!(flattened["assignees"].as_array().unwrap().len(), 0);
}

#[test]
fn log_entry_with_metadata() {
    let flattener = Flattener::new();

    let log_entry = json!({
        "timestamp": "2024-01-15T10:30:00Z",
        "level": "ERROR",
        "message": "Database connection failed",
        "context": {
            "service": "api-gateway",
            "environment": "production",
            "region": "us-east-1",
            "error": {
                "type": "ConnectionError",
                "code": 500,
                "details": "Connection timeout after 30s"
            }
        },
        "tags": ["database", "critical"]
    });

    let flattened = flattener.flatten(&log_entry);

    assert_eq!(flattened["timestamp"], "2024-01-15T10:30:00Z");
    assert_eq!(flattened["level"], "ERROR");
    assert_eq!(flattened["message"], "Database connection failed");
    assert_eq!(flattened["context.service"], "api-gateway");
    assert_eq!(flattened["context.environment"], "production");
    assert_eq!(flattened["context.region"], "us-east-1");
    assert_eq!(flattened["context.error.type"], "ConnectionError");
    assert_eq!(flattened["context.error.code"], 500);
    assert_eq!(
        flattened["context.error.details"],
        "Connection timeout after 30s"
    );
    assert_eq!(flattened["tags"][0], "database");
    assert_eq!(flattened["tags"][1], "critical");
}

#[test]
fn custom_separator_integration() {
    let flattener = Flattener {
        separator: "_",
        ..Default::default()
    };

    let data = json!({
        "user": {
            "profile": {
                "name": "Alice"
            }
        }
    });

    let flattened = flattener.flatten(&data);

    assert_eq!(flattened["user_profile_name"], "Alice");
    assert!(flattened.get("user.profile.name").is_none());
}

#[test]
fn preserve_arrays_integration() {
    let flattener = Flattener {
        preserve_arrays: true,
        ..Default::default()
    };

    let data = json!({
        "items": [
            {"id": 1, "name": "First"},
            {"id": 2, "name": "Second"}
        ]
    });

    let flattened = flattener.flatten(&data);

    assert_eq!(flattened["items.0.id"], 1);
    assert_eq!(flattened["items.0.name"], "First");
    assert_eq!(flattened["items.1.id"], 2);
    assert_eq!(flattened["items.1.name"], "Second");
}
