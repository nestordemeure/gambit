
library(ggplot2)

# dataset
path <- "/home/nestor/Documents/programming/F/symbolicRegression/src/demo/input/vertebral_column_3C.csv"
data <- read.csv(path)
label <- data$class

# variables
pelvic_incidence <- data$pelvic_incidence
pelvic_tilt <- data$pelvic_tilt
lumbar_lordosis_angle <- data$lumbar_lordosis_angle
sacral_slope <- data$sacral_slope
pelvic_radius <- data$pelvic_radius
degree_spondylolisthesis <- data$degree_spondylolisthesis

# formula
x <- pelvic_radius + sacral_slope
y <- degree_spondylolisthesis - pelvic_radius
dataFormula <- data.frame(x=x, y=y, label=label)
X11()
ggplot(data=dataFormula, aes(x=x, y=y, color=label)) + geom_point()

# pca
dataPCA <- data
dataPCA$class <- NULL
pca <- prcomp(dataPCA, center=FALSE, scale = FALSE)
dataPCA <- as.data.frame(pca$x[,1:2])
dataPCA$label <- label 
X11()
ggplot(data=dataPCA, aes(x=PC1, y=PC2, color=label)) + geom_point()
