# Based Scraper 🦀 

## This is a **RUST** program that compiles into a **binary** and scrapes job postings from various websites. The program first defines a set of modules, each representing a company to scrape job postings from.  Each module has a `scrape()` function that runs **asynchronously** and returns a result.

> Să tragem date cu Trag Date -- AYYY LMAO :trollface:

![](based_scrapper.png)

## :rocket: Features 
### Main features 
- Less SOY 
- More based
- Blazing fast
- Asynchronous

 The program currently scrapes jobs from the following websites:
- BCR
- Vodafone
- Decathlon
- Samsung
- Allianz
- Auchan
- Autonom
- BRD
- Draexlmaier
- Enel
- EON
- FedEx
- Generali
- HM
- Kaufland
- Medicover
- Linde
- Zentiva

## :scroll: Output
For each company it creates a JSON object containing: 
- Job title
- URL link to the job post
- Company name
- Job location
- Country where the job is located

The output is written to a separate JSON file for each company.

## :factory: Requirements
If you want to build the program from source, you will need the following dependencies:
- 🦀 Rust 
- 📦 Cargo

## 💾 Installation & Building
The program can be cloned from the repository using git. 

```
git clone https://github.com/peviitor-ro/based_scrapper.git
```

After installation, navigate to the root directory and build the program using the command:

```
cargo build --release
```

## ⌨️ Usage

```bash
cd target/release
./based_scrapper
```
Or you can copy the binary to your `/usr/bin` directory and run it from anywhere with the command:

```bash
based_scrapper
```

## :ninja: Author
This program is written by [Trag Date](https://github.com/tragdate) and documented by [docuTron](https://github.com/tragdate/docuTron)

## License
This program is licensed under the GPL-3.0 License

## Proudly made with [neovim](https://neovim.io/) using only the 🐚 terminal and a ⌨️ keyboard

## TODO
- [ ] Add more companies
- [ ] Add more features
- [ ] Add more documentation
- [ ] Handle errors better
- [ ] Templates for the 4 kinds of scrapping
- [ ] Organize the code better (currently spaghetti code)
- [ ] Integrate a UID system
- [ ] Send results to API
- [ ] Implement update API algorithm 
