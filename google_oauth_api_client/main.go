package main

import (
    "encoding/json"
    "context"
    "fmt"
    "os"
    "google.golang.org/api/idtoken"
)

func main() {


    tokenString := os.Getenv("GOOGLE_OAUTH_EXEC_INPUT_TOKEN")
    clientId := os.Getenv("GOOGLE_OAUTH_EXEC_CLIENT_ID")

    payload, err := idtoken.Validate(context.Background(), tokenString, clientId)

    if err != nil {
        panic(err)
    }

    json, err := json.Marshal(payload.Claims)
    if err != nil {
        panic(err)
    }

    fmt.Println(string(json))

}
