# cr_tiler_rs
A Piano Tiles clone that includes a game service server that lets users store their high scores, and the game itself.

### Features:
- Timing based input
- Leaderboards system
- Hard mode, introducing more varied tile speeds
- Modify the number of input slots

### Running a leaderboards server:
#### Setup:
```bash
git clone https://github.com/CoryRobertson/cr_tiler_rs.git
cd cr_tiler_rs
docker-compose build
docker-compose up
```
#### Connecting to the server:
Open the client, and connect to the ip address of the docker container, and the port 8114.
E.g. "192.168.1.86:8114"
Be sure to click the "Play Online ?" checkbox such that it is darkened. If connection is successful, a globe icon without a red cancellation sign over it should appear in the top right. After the player finishes a session, their score should be uploaded along with their name.

### Main Menu:
![Image of the tile games main menu](https://raw.githubusercontent.com/CoryRobertson/cr_tiler_rs/main/images/MainMenu.png)
### In game:
![Image of the in-game portion of the tile game on normal mode](https://raw.githubusercontent.com/CoryRobertson/cr_tiler_rs/main/images/NormalMode.png)
### In game Hard Mode:
![Image of the in-game portion of the tile game on hard mode](https://raw.githubusercontent.com/CoryRobertson/cr_tiler_rs/main/images/HardMode.png)
### End Screen:
![Image of the games end screen](https://raw.githubusercontent.com/CoryRobertson/cr_tiler_rs/main/images/EndScreen.png)

###### ASSets by Tom 