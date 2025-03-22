nightly_{{name}} = { optional = true, package = "rg-lib", git = "ssh://git@galaxy-sec.org/free/galaxy/rg-lib.git" , branch = "nightly" }
beta_{{name}}    = { optional = true, package = "rg-lib", git = "ssh://git@galaxy-sec.org/free/galaxy/rg-lib.git" , branch = "beta" }
master_{{name}}  = { optional = true, package = "rg-lib", git = "ssh://git@galaxy-sec.org/free/galaxy/rg-lib.git" , master = "master" }
release_{{name}} = { optional = true, package = "rg-lib", git = "ssh://git@galaxy-sec.org/free/galaxy/rg-lib.git" , tag    = "{{tag}}" }
