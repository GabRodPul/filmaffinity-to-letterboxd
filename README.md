# filmaffinity-to-letterboxd
A simple webscraping tool to convert a FilmAffinity profile to Letterboxd's import format.

## Requirements
- Rust (in case you want to install using `cargo`).

## How to install
### Using cargo
> [!NOTE]
> TODO: Add how to install using `cargo`

### Using provided binaries
> [!NOTE]
> TODO: Add how to install using provided binaries


## How to use
- Go to your FilmAffinity profile and copy your user ID number from the link.
<p align="center"><img width="652" height="46" alt="image" src="https://github.com/user-attachments/assets/371fd33b-db90-4035-8cc5-2bc86ee52ff9"   /></p>

- Run the program, specifying at least user ID (`-u`). Wait for it to complete. By default, it'll fetch up to 1000 pages of content. If by any case, you have more, add `-p <page_count>`. For the full list of flags, run `filmaffinity-to-letterboxd -h`.
```
filmaffinity-to-letterboxd -u <user_id>
```

> [!WARNING]
> The app will delay the next request after processing all data by a random integral range of [1, 3].
> Although as far as testing goes, no delay hasn't caused any issues, better to be safe than sorry.
> To disable it, add `-d` to the flags.
<img width="1472" height="524" alt="image" src="https://github.com/user-attachments/assets/957f8188-3af7-46a9-b58e-766b8a40ee6a" />


- Once you have your file ready (by default, saved as `fa-to-letterboxd-result.csv`), head to [Letterboxd's Import page](https://letterboxd.com/import/) and upload it by clicking "SELECT A FILE". It'll take a while to process it all.
<p align="center"><img width="1026" height="513" alt="image" src="https://github.com/user-attachments/assets/e9ffdf8b-b25a-4239-8223-c24ce02cf226" /></p>
<p align="center"><img width="677" height="312" alt="image" src="https://github.com/user-attachments/assets/096f17ee-bf8e-42b2-9fbb-85d647629a14"  /></p>

- When finished, you'll be redirected to a page where you can confirm the export. To reduce the number of entries on screen, click "Hide successful matches".
<p align="center"><img width="1006" height="747" alt="image" src="https://github.com/user-attachments/assets/91b57232-4e9e-4769-a8ae-30f46f93919f" /></p>
<p align="center"><img width="972" height="184" alt="image" src="https://github.com/user-attachments/assets/012b1e1b-c398-4bce-8466-278024982472"  /></p>

> [!WARNING]
> When importing, not all entries will get exported. This may be due to an entry having a different name in Letterboxd or it's not there (series or individual chapters from series are usually the ones missing).
> TODO: Add some sort of table for title conversions/pattern matching.
<p align="center"><img width="972" height="539" alt="image" src="https://github.com/user-attachments/assets/2facc8fa-9267-4d80-a865-429c055e6b1c" /></p>
