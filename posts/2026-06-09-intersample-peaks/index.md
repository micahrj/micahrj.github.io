+++
title = "Intersample peaks are unbounded"
date = "2026-06-09T00:00:00-05:00"
+++

In digital audio, we work with discrete sequences of values, but we generally treat these values as implicitly corresponding to samples of some underlying continuous-time signal. The correspondence between the two is given by the [Nyquist-Shannon sampling theorem](https://en.wikipedia.org/wiki/Nyquist%E2%80%93Shannon_sampling_theorem), which gives conditions under which a continuous-time signal can be reconstructed exactly from a sequence of regularly spaced samples. The reconstruction process, known as sinc interpolation, is given by the [Whittaker-Shannon interpolation formula](https://en.wikipedia.org/wiki/Whittaker%E2%80%93Shannon_interpolation_formula):

\[ x(t) = \sum_{n=-\infty}^{\infty} x[n] \, \mathrm{sinc} \left( \frac{t - nT}{T} \right) \]

where \(T\) is the sampling interval.

- sinc interpolation yields a function which exactly passes through the sampled values
- however, it is not monotonic, meaning that the values of the function in between the sample points may not stay between those two sample points. in other words, there can be "overshoot".
- 

One notable property of sinc interpolation is that it is not monotonic 

<!--excerpt-->
