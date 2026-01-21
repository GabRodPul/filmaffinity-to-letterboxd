# filmaffinity-to-letterboxd
A simple webscraping tool to convert a FilmAffinity profile to Letterboxd's import format.

## Requirements
- Preferred webdriver implementation ([ChromeDriver](https://developer.chrome.com/docs/chromedriver/downloads), [GeckoDriver](https://developer.chrome.com/docs/chromedriver/downloads))
- Rust (in case you want to install using `cargo`).

## How to install
### Using cargo
> [!NOTE]
> TODO: Add how to install using `cargo`

### Using provided binaries
> [!NOTE]
> TODO: Add how to install using provided binaries


## How to use
- Start your WebDriver instance. For example, with ChromeDriver:
```
chromedriver --port=<webdriver_port>`
```
- Go to your FilmAffinity profile and copy your user ID number from the link (https://www.filmaffinity.com/en/userratings.php?**user_id={user_id}**&p=1&orderby=4&chv=grid) and scroll down and see how many pages your profile has. For example, 14.
<img width="801" height="124" alt="image" src="https://github.com/user-attachments/assets/20e15b47-9ee0-421d-8a3e-a4d52f07cea6" />
 
- Run the program, specifying at least user ID (`-u`) and WebDriver port (`-p`).
```
filmaffinity-to-letterboxd -u <user_id> -p <webdriver_port>
```

> [!NOTE]
> TODO: Add in the process of importing output into Letterboxd
