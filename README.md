<div align="center">
<h1>vsh</h1>
A Blazingly fast shell made in Rust 🦀
</div>

#### Why make another shell?

[Nushell](https://github.com/nushell/nushell/) which is the current leading rust shell, is very opinionated. It brings a lot to the table for someone who just wants a shell but in rust. Namely, a new `ls` command, new scripting experiece etc. What is missing is just bash written in rust and vsh is here to deliver that. The planned scripting language will be a interchangable with bash and all of its features so that people don't feel *homesick* or dropped into a new space when they start using vsh. As the for shell prompt I plan to expand upon it to add plugins to accept a wide array of custom plugins all written in vsh. Till then feel free to contribute yourself!

#### Fonts Needed

Now that the prompt has been given a complete overhaul, you will need need to install nerd fonts to run `vsh` properly. [Click here](https://github.com/ryanoasis/nerd-fonts) to choose your own nerd font, no need to worry as the font that you are used to using should probably have its own nerd font counterpart. I recommend using [Jetbrains Mono Nerd font](https://github.com/ryanoasis/nerd-fonts/blob/master/patched-fonts/JetBrainsMono/Ligatures/Regular/complete/JetBrains%20Mono%20Regular%20Nerd%20Font%20Complete%20Mono.ttf) to get started!

#### Customization

To customize your prompt you have to edit the `.vshrc.json` file in your home directory.
The file is created when you first open `vsh`.
When initialized the file contains an emtpy json object, i.e:
```
{

}
```
##### Themes
There are two themes with the following properties
- Classic: `double: boolean value` and `character: string value`.
- Modern: `double: boolean value`, `character: string value`, `color: List with 3 elements`, `text_color: List with 3 elements`
###### Double
**defualt value:** `False` 

Determines wether the prompt will be double lined or single lined
eg:
```json
{
	"double": "false"
}
```
Keep in mind that you have to enter `true` or `false` as string i.e inside double quotes

###### Character
**default value**: `λ`

The character that you will have in the starting of the prompt.
###### Color
**default**: `[109, 152, 134]`

Determines background color of modern shell prompt. Eg:
```json
{
	"color": [255, 255, 255]
}
```
###### Text Color
**default**: `[33, 33, 33]`

Determines text color of modern shell prompt. Eg:
```json
{
	"color": [255, 255, 255]
}
```
#### Example Config file
This is the config file personally used by me:
```json
{
	"style": "modern",
	"color": [115, 147, 179]
}
```
As you can see customizing is not that hard and doesn't require too much code. I will be adding more and more into the level of customization possible!
#### Roadmap

- [x] Proper Prompt
- [x] Run commands
- [x] Exit with Ctrl+C & Ctrl+D via Rustyline
- [x] Good looking prompt
- [x] Multiple Commands
- [x] Command History
- [x] Prompt Customization
- [ ] Install Script
- [ ] Piping
- [ ] Command Completion
- [ ] `vsh` Scripting language :eyes:
- [ ] Custom `ls` command
- [ ] Intergration with `git`, `node` and `cargo`
- [ ] Customization via `.vshrc`
- [ ] Plugin Support (Yikes!)
