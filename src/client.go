package main

import (
	"context"
	"crypto/tls"
	"fmt"
	"golang.org/x/net/http2"
	"net"
	"net/http"
	"os"
	"time"
)

func checkErr(err error, msg string) {
	if err == nil {
		return
	}
	fmt.Printf("ERROR: %s: %s\n", msg, err)
	os.Exit(1)
}

func main() {
	HttpClientExample(os.Args[1]	)
	//RoundTripExample()
}



func RoundTripExample(url string) {
	req, err := http.NewRequest("GET", url, nil)
	checkErr(err, "during new request")

	ctx, cancel := context.WithTimeout(context.Background(), time.Second*10)
	defer cancel()

	tr := &http2.Transport{
		AllowHTTP: true,
		DialTLS: func(network, addr string, cfg *tls.Config) (net.Conn, error) {
			return net.Dial(network, addr)
		},
	}

	req.WithContext(ctx)
	resp, err := tr.RoundTrip(req)
	checkErr(err, "during roundtrip")

	fmt.Printf("RoundTrip Proto: %d\n", resp.ProtoMajor)
}

func HttpClientExample(url string) {
	client := http.Client{
		Transport: &http2.Transport{
			AllowHTTP: true,
			DialTLS: func(network, addr string, cfg *tls.Config) (net.Conn, error) {
				return net.Dial(network, addr)
			},
		},
	}

	req, err := http.NewRequest("GET", url, nil)
	checkErr(err, "during get")
	req.Header.Add("Cookie", "CONSENT=PENDING+097; CONSENT2=PENDING+097; CONSENT3=PENDING+097")
	resp, err := client.Do(req)
	// resp, err := client.Get(url)

	checkErr(err, "during get")

	fmt.Printf("Client Proto: %d\n", resp.ProtoMajor)
}
