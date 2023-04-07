#! /usr/bin/env python

from redfish import redfish_client

CREDS = {
    "base_url": "https://bmc-oahu10000",
    "username": "admin",
    "password": "password",
}



if __name__ == "__main__":
    cnx = redfish_client(**CREDS, default_prefix="/redfish/v1/")
    cnx.login(**CREDS, auth="session")
    resp = cnx.get("/redfish/v1/systems/1", None)
    print(resp)
