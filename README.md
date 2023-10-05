## A simple webdav server written by Rust.


### How to use?

1. download the bin file in the release page.
2. create a config json file:
   ```json
    {
      "port": 9988, // any port not used by other application
      "mode": "",  // default is "", if your webdav client support 416 http code, you can set it "dev"
      "log": "info", // log level, info/warn/error
      // user rules, support guest user(empty user and empty password) and basic auth user
      "rules": [
        {
          "path": "/Users/guest/", // shared path
          "permission": "R", // current user permission, "RWD" ==> READ/WRITE/DELETE
          "server_prefix": "/guest" // access url prefix, e.g. "http://192.168.2.2:9988/webdav-guest"
        },
        {
          "user": "usera",
          "password": "123",
          "path": "/Users/a/",
          "permission": "RW",
          "server_prefix": "/a"
        },
        {
          "user": "userb",
          "password": "456",
          "path": "/Users/b/",
          "permission": "RWD",
          "server_prefix": "/b"
        }
      ]
    }
   ```
3. run the server: `./rhfs "config=/path/to/config.json"`
