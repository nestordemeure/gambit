
library(ggplot2)

# iris dataset
petal_width <- iris$Sepal.Width
petal_length <- iris$Petal.Length
sepal_width <- iris$Sepal.Width
sepal_length <- iris$Sepal.Length
label <- iris$Species

# formula
x <- petal_width
y <- (petal_length*petal_length) / (sqrt(sepal_width) * sepal_length)
data <- data.frame(x=x, y=y, label=label)

# display
X11()
ggplot(data=data, aes(x=x, y=y, color=label)) + geom_point()