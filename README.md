# Yarn wrapper generator utility

This tool generates wrapper classes using [yarn](https://github.com/FabricMC/yarn) mappings. It can be used as a library or a standalone Fabric mod.

Output: [**Yarnwrap**](https://github.com/FabricCore/yarnwrap)

## Motivation

The obfuscated nature of Minecraft prevents runtime methods of accessing code in game. This project is created to expose more features of the game to scripting runtimes.

## Specification

### Conversion
The constructor of the wrapper class takes the source class as argument.
```java
SourcePlayer p = mc.player;
WrappedPlayer wp = new WrappedPlayer(p);
```

The wrapped object is stored in public field `wrapperContained`.
```java
WrappedPlayer wp = new WrappedPlayer(mc.player);
SourcePlayer p = wp.wrapperContained;
```

### Methods
Methods can be accessed same as how it is accessed in its source class.

### Fields
Fields can be accessed as a function by its original name.
```java
int x1 = source.x;
int x2 = wrapped.x();
x1 == x2 // true
```

And using a similar mechanism for writing to fields.
```java
source.x = 123;
wrapped.x(123);
```

### Inheritance

Since yarn mappings does not contain inheritance descriptors, inheritance is not represented in the generated code. For instance, using the source classes.
```java
client.Player p;
p.getBlockX(); // .getBlockX() is in client.Entity, not Player
```
In wrapper classes, you can only use classes that are strictly defined in `client.player`. Alternatively, you may manually convert between classes.
```java
client.PlayerWrapped p;
client.EntityWrapped e = new EntityWrapped(p);
e.getBlockX(); // .getBlockX() is in client.Entity, so we can access it from `e`
```

## Build

For Minecraft versions without releases, you can build it yourself instead.

### Prerequisite

1. `yarn-wrapper-gen` installed on system.
```sh
cargo install --git 'https://github.com/FabricCore/yarn-wrapper-gen'
```
2. `gradlew-commentator` installed on system.
```sh
cargo install --git 'https://github.com/FabricCore/gradlew-commentator'
```
3. `yarn` mappings accessible on system.
```sh
git clone 'https://github.com/FabricMC/yarn' ~/Downloads/yarn
```
4. Specify Minecraft version. Browse [`yarn` commits](https://github.com/FabricMC/yarn/commits) to find the specific game version.
```sh
git checkout [commit-hash]
```
5. Create or open an existing Minecraft mod project (FabricMC + Gradle) make sure your code compiles without error, add the following line to `gradle.build`.
```groovy
allprojects {
  gradle.projectsEvaluated {
    tasks.withType(JavaCompile) {
        options.compilerArgs << "-Xmaxerrs" << "100000000"
    }
  }
}
```

### Generation

1. Use `yarn-wrapper-gen` to generate uncleaned wrapper classes.
```sh
yarn-wrapper-gen ~/Downloads/yarn/mappings/net ~/Downloads/yarnwrap 'com.example.package.yarnwrap'
```
> - The first argument points to `/mappings/net` in the yarn repository.
> - The second argument points to the output path for the generated files.
> - The third argument is package name for the generated code in your project.
>
> Note that arguments in groups of 2 remaps the parts of the qualifying class name, the `yarnwrap` library remaps `yarnwrap.net.minecraft` to `yarnwrap`.
> ```sh
> yarn-wrapper-gen ~/Downloads/yarn/mappings/net ~/Downloads/yarnwrap 'yarnwrap' 'yarnwrap.net.minecraft' 'yarnwrap'
> ```
2. Copy the generated files to your project, at the specified location (argument 3). Again **make sure that your code compiles without errors**.
3. Change directory to your project, and use `gradlew-commentator` to clean your code to a compilable state. This process runs `./gradlew check` multiple times and may take up to 10 minutes.
```sh
gradlew-commentator
```
> Note that if `gradlew-commentator` stop during execution, and `./gradlew check` shows the code still contain errors, clean compiler cache with `./gradlew clean` and run it again.
