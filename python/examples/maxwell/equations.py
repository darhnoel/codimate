"""The equations as the film writes them, in differential form, as LaTeX.

`∂` is typed directly: the LaTeX bridge turns `\\partial` into a Typst symbol the
installed Typst does not have."""

# The four equations as the film writes them, in differential form.
GAUSS = r"\nabla\cdot\mathbf{E}=\frac{\rho}{\varepsilon_0}"
NO_MONOPOLE = r"\nabla\cdot\mathbf{B}=0"
FARADAY = r"\nabla\times\mathbf{E}=-\frac{∂\mathbf{B}}{∂ t}"
AMPERE = r"\nabla\times\mathbf{B}=\mu_0\mathbf{J}"
MAXWELL_TERM = r"+\mu_0\varepsilon_0\frac{∂\mathbf{E}}{∂ t}"
AMPERE_ONLY_E = r"\nabla\times\mathbf{B}=\mu_0\varepsilon_0\frac{∂\mathbf{E}}{∂ t}"
