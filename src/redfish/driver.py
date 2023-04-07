#! /usr/bin/env python

from redfish import redfish_client

CREDS0 = {
    "base_url": "https://bmc-oahu10000",
    "username": "admin",
    "password": "password",
}

CREDS1 = {
    "username": "admin",
    "password": "password",
}




if __name__ == "__main__":
    cnx = redfish_client(**CREDS0, default_prefix="/redfish/v1/")
    cnx.login(**CREDS1, auth="session")
    resp = cnx.get("/redfish/v1/systems/1", None, **CREDS1)
    print(resp)
