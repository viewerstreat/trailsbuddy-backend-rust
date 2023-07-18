#!/bin/bash

mongosh "mongodb+srv://treatviewersdbusr:48MtSlonDyPbazuq@treatviewers.deppn.mongodb.net/treatviewerstest?authSource=admin" < test_preparation.js


# docker run -d --name mongotestdb -p 27017:27017 -e MONGO_INITDB_ROOT_USERNAME=sibaprasad -e MONGO_INITDB_ROOT_PASSWORD=password1 mongo
# docker exec -it mongotestdb bash
# mongosh --username sibaprasad --password password1
# mongodb://sibaprasad:password1@0.0.0.0:27017/treatviewerstest
