"""Web framework enumerations."""

from enum import Enum

from .programming_languages import ProgrammingLanguage as Language


class WebFramework(Enum):
    """A web framework enum that also stores its associated Language.

    Each member = (Language, framework_name).
    """

    # Python
    django = (Language.python, "django")
    flask = (Language.python, "flask")
    fastapi = (Language.python, "fastapi")
    starlette = (Language.python, "starlette")
    tornado = (Language.python, "tornado")
    pyramid = (Language.python, "pyramid")
    sanic = (Language.python, "sanic")
    aiohttp = (Language.python, "aiohttp")
    falcon = (Language.python, "falcon")
    masonite = (Language.python, "masonite")
    bottle = (Language.python, "bottle")
    quart = (Language.python, "quart")

    # JavaScript / TypeScript
    express = (Language.javascript, "express")
    nextjs = (Language.javascript, "nextjs")
    nuxtjs = (Language.typescript, "nuxtjs")
    nestjs = (Language.typescript, "nestjs")
    remix = (Language.typescript, "remix")
    astro = (Language.typescript, "astro")
    sveltekit = (Language.typescript, "sveltekit")
    koa = (Language.javascript, "koa")
    hapi = (Language.javascript, "hapi")
    adonisjs = (Language.typescript, "adonisjs")
    meteor = (Language.javascript, "meteor")

    # Ruby
    rails = (Language.ruby, "rails")
    sinatra = (Language.ruby, "sinatra")
    hanami = (Language.ruby, "hanami")
    padrino = (Language.ruby, "padrino")

    # PHP
    laravel = (Language.php, "laravel")
    symfony = (Language.php, "symfony")
    codeigniter = (Language.php, "codeigniter")
    yii = (Language.php, "yii")
    cakephp = (Language.php, "cakephp")
    drupal = (Language.php, "drupal")
    wordpress = (Language.php, "wordpress")

    # Java / JVM
    spring = (Language.java, "spring")
    spring_boot = (Language.java, "spring_boot")
    micronaut = (Language.java, "micronaut")
    quarkus = (Language.java, "quarkus")
    sparkjava = (Language.java, "sparkjava")
    vertx = (Language.java, "vertx")
    grails = (Language.groovy, "grails")
    play = (Language.scala, "play")

    # Go
    gin = (Language.go, "gin")
    fiber = (Language.go, "fiber")
    echo = (Language.go, "echo")
    beego = (Language.go, "beego")
    revel = (Language.go, "revel")
    chi = (Language.go, "chi")

    # Rust
    actix = (Language.rust, "actix")
    rocket = (Language.rust, "rocket")
    warp = (Language.rust, "warp")
    axum = (Language.rust, "axum")
    poem = (Language.rust, "poem")
    salvo = (Language.rust, "salvo")
    perseus = (Language.rust, "perseus")
    leptos = (Language.rust, "leptos")

    # .NET
    aspnet = (Language.csharp, "aspnet")
    blazor = (Language.csharp, "blazor")

    # Elixir
    phoenix = (Language.elixir, "phoenix")

    # Clojure
    ring = (Language.clojure, "ring")
    luminus = (Language.clojure, "luminus")

    # Deno
    fresh = (Language.javascript, "fresh")
    aleph = (Language.javascript, "aleph")

    # Kotlin
    ktor = (Language.kotlin, "ktor")

    # Swift
    vapor = (Language.swift, "vapor")

    # Dart
    dart_frog = (Language.dart, "dart_frog")

    # Haskell
    yesod = (Language.haskell, "yesod")
    scotty = (Language.haskell, "scotty")
    servant = (Language.haskell, "servant")

    # Lua
    lapis = (Language.lua, "lapis")
    sailor = (Language.lua, "sailor")

    # Perl
    dancer = (Language.perl, "dancer")
    mojolicious = (Language.perl, "mojolicious")
    catalyst = (Language.perl, "catalyst")

    # Racket
    web_server = (Language.racket, "web_server")

    # Scala
    finatra = (Language.scala, "finatra")
    scalatra = (Language.scala, "scalatra")

    # OCaml
    ocsigen = (Language.ocaml, "ocsigen")
    dream = (Language.ocaml, "dream")

    # Nim
    jester = (Language.nim, "jester")

    # Zig
    zap = (Language.zig, "zap")

    # Common Lisp
    caveman2 = (Language.lisp, "caveman2")

    # Julia
    genie = (Language.julia, "genie")

    # Erlang
    cowboy = (Language.erlang, "cowboy")

    # ColdFusion
    coldfusion = (Language.cobol, "coldfusion")  # loosely typed fallback

    # ---------------- Utility properties ----------------
    @property
    def language(self) -> Language:
        """Get the programming language."""
        return self.value[0]

    @property
    def framework(self) -> str:
        """Get the framework name."""
        return self.value[1]

    def __str__(self) -> str:
        """Return string representation."""
        return self.framework
