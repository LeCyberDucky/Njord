import yaml
import pandas as pd
import matplotlib.pyplot as plt

with open("Data/Data.yaml", "r") as stream:
    data_nested = yaml.safe_load(stream)

data = []
for sublist in data_nested:
    for item in sublist:
        data.append(item)

data = pd.json_normalize(data)
data[["secs_since_epoch", "nanos_since_epoch"]] = data[["secs_since_epoch", "nanos_since_epoch"]].shift(-1)
data = data.dropna().reset_index(drop = True)
data["time"] = pd.to_datetime(data["secs_since_epoch"], unit = "s") + pd.to_timedelta(data["nanos_since_epoch"], unit = "ns")
data.drop(["secs_since_epoch", "nanos_since_epoch"], axis = "columns")
# Note that the timezone is probably off, but eh, working with time is a mess

plt.plot(data["time"], data["acceleration.z"])


plt.plot(data["time"], data["acceleration.z"] - data["acceleration.z"].mean())


plt.plot(data["time"], data["temperature"])








# import altair as alt
# alt.renderers.enable("svg")
# alt.Chart(data[0:1000]).mark_point().encode(
#     x = "time",
#     y = "temperature"
# )
