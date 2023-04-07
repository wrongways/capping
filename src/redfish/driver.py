from redfish import redfish_client

CREDS = {
    "base_url": "https://bmc-oahu10000",
    "username": "admin",
    "password": "password",
}




cnx = redfish_client(**CREDS)
