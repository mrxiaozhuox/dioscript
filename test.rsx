@name = "hello rsx";
@dark = false;

@root_class = "bg-gray-800";

@data = ["u1", "u2", "u3"];

if @name == "hello world" || (true == true) {
	return "dark theme unsupport.";
} else {
	return "SB";
}

@enable_sub_componet = true;
@sub_component = div { "Hello world" };

return div {
	class: @root_class,
	h1 { "title" },
	h2 { name: "hello" },
	div {
		class: "prose",
		img {
			src: "avatar.jpg"
		},
		span { "YuKun Liu" },
		if @enable_sub_conponent { 
			return @sub_components;
		},
	}
};
