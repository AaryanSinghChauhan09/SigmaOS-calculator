    # SigmaOS-calculator

    Scientific calculator engine

    Part of the **SigmaOS** sovereign operating-system ecosystem — a multi-repository
    distribution modelled on Linux Mint and Omarchy Linux. See the parent
    [`SigmaOS`](https://github.com/AaryanSinghChauhan09/SigmaOS) repository for the
    full component index.

    ## Upstream inspirations
    - https://github.com/omacom/omacalc

    ## Design principles applied
    - **Abstraction**
- **Least Astonishment**
- **Defensive Programming**
- **Fail-Fast**
- **KISS**
- **Separation of Concerns**

    ## Usage
    Verbs: `eval, repl`

    ```
    $ sigma-calc eval "2 + 3 * 4"
14
$ sigma-calc eval "sqrt(16) + sin(0)"
4.0
$ sigma-calc repl

    ```

    ## Build & test
    ```
    make build
    make test
    ```

    ## Licence
    GNU General Public License v3.0 (FOSS).
