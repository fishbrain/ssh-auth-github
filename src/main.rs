extern crate reqwest;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use reqwest::header;
use std::fs::File;

#[derive(Deserialize, Debug)]
struct Response {
    data: Data,
}

#[derive(Deserialize, Debug)]
struct Team {
    members: TeamMemberConnection,
}

#[derive(Deserialize, Debug)]
struct Organization {
    team: Team,
}

#[derive(Deserialize, Debug)]
struct Data {
    organization: Organization,
}

#[derive(Deserialize, Debug)]
struct TeamMemberConnection {
    nodes: Vec<User>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct User {
    login: String,
    public_keys: PublicKeyConnection,
}

#[derive(Deserialize, Debug)]
struct PublicKeyConnection {
    nodes: Vec<PublicKey>,
}

#[derive(Deserialize, Debug)]
struct PublicKey {
    key: String,
}

#[derive(Serialize, Debug)]
struct Query {
    query: String,
}

#[derive(Deserialize, Debug)]
struct Config {
    token: String,
    organization: String,
    team: String,
}

fn query(token: &str, organization: &str, team: &str) {
    let client = reqwest::Client::new();

    let query = Query {
        query: format!(
            r#"
query {{
  organization(login: "{}") {{
    team(slug: "{}") {{
      members(first: 100) {{
        nodes {{
          login
          publicKeys(first: 100) {{
            nodes {{
              key
            }}
          }}
        }}
      }}
    }}
  }}
}}
"#,
            &organization, &team
        ),
    };

    let mut resp = client
        .post("https://api.github.com/graphql")
        .json(&query)
        .header(header::Authorization(header::Bearer {
            token: token.to_string(),
        }))
        .send()
        .unwrap();

    let data: Response = serde_json::from_str(&resp.text().unwrap()).unwrap();

    for user in &data.data.organization.team.members.nodes {
        for key in &user.public_keys.nodes {
            println!("{} {}", &key.key, user.login);
        }
    }
}

fn main() {
    let config_file = File::open("/etc/ssh-auth-github.json").expect("Open configuration file");
    let config: Config  = serde_json::from_reader(config_file).expect("Parsing configuration file");

    query(&config.token, &config.organization, &config.team);
}
